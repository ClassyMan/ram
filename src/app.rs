use anyhow::Result;

use crate::collector::{
    self, HardwareInfo, MemInfo, PsiSnapshot, VmRates, VmStatSnapshot,
};
use crate::ring_buffer::RingBuffer;

const FAST_REFRESH_MS: u64 = 25;
const FAST_SCROLLBACK_SECS: u64 = 3;

pub struct App {
    pub alloc_history: RingBuffer,
    pub free_history: RingBuffer,
    pub swapin_history: RingBuffer,
    pub swapout_history: RingBuffer,
    pub fault_history: RingBuffer,
    pub major_fault_history: RingBuffer,
    pub psi_some_history: RingBuffer,
    pub psi_full_history: RingBuffer,

    pub latest_info: Option<MemInfo>,
    pub latest_rates: Option<VmRates>,
    pub latest_psi: Option<PsiSnapshot>,
    pub hardware: HardwareInfo,
    pub should_quit: bool,
    pub scrollback_secs: u64,
    pub fast_mode: bool,
    pub refresh_ms: u64,
    normal_refresh_ms: u64,
    normal_scrollback_secs: u64,
    prev_vmstat: Option<VmStatSnapshot>,
}

impl App {
    pub fn new(refresh_ms: u64, scrollback_secs: u64) -> Self {
        let capacity = ((scrollback_secs * 1000) / refresh_ms) as usize;
        let hardware = collector::read_hardware_info();
        Self {
            alloc_history: RingBuffer::new(capacity),
            free_history: RingBuffer::new(capacity),
            swapin_history: RingBuffer::new(capacity),
            swapout_history: RingBuffer::new(capacity),
            fault_history: RingBuffer::new(capacity),
            major_fault_history: RingBuffer::new(capacity),
            psi_some_history: RingBuffer::new(capacity),
            psi_full_history: RingBuffer::new(capacity),
            latest_info: None,
            latest_rates: None,
            latest_psi: None,
            hardware,
            should_quit: false,
            scrollback_secs,
            fast_mode: false,
            refresh_ms,
            normal_refresh_ms: refresh_ms,
            normal_scrollback_secs: scrollback_secs,
            prev_vmstat: None,
        }
    }

    pub fn chart_capacity(&self) -> usize {
        self.alloc_history.capacity()
    }

    pub fn toggle_fast_mode(&mut self) {
        self.fast_mode = !self.fast_mode;

        let (new_refresh, new_scrollback) = if self.fast_mode {
            (FAST_REFRESH_MS, FAST_SCROLLBACK_SECS)
        } else {
            (self.normal_refresh_ms, self.normal_scrollback_secs)
        };

        self.refresh_ms = new_refresh;
        self.scrollback_secs = new_scrollback;

        let capacity = ((new_scrollback * 1000) / new_refresh) as usize;
        self.alloc_history = RingBuffer::new(capacity);
        self.free_history = RingBuffer::new(capacity);
        self.swapin_history = RingBuffer::new(capacity);
        self.swapout_history = RingBuffer::new(capacity);
        self.fault_history = RingBuffer::new(capacity);
        self.major_fault_history = RingBuffer::new(capacity);
        self.psi_some_history = RingBuffer::new(capacity);
        self.psi_full_history = RingBuffer::new(capacity);
        self.prev_vmstat = None;
        self.latest_rates = None;
    }

    pub fn refresh_rate(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.refresh_ms)
    }

    pub fn tick(&mut self) -> Result<()> {
        let info = collector::read_meminfo()?;
        let vmstat = collector::read_vmstat()?;
        let psi = collector::read_psi().ok();

        if let Some(prev) = &self.prev_vmstat {
            let interval_secs = self.refresh_ms as f64 / 1000.0;
            let rates = VmRates::from_deltas(prev, &vmstat, interval_secs);

            self.alloc_history.push(rates.alloc_mb_per_sec);
            self.free_history.push(rates.free_mb_per_sec);
            self.swapin_history.push(rates.swapin_mb_per_sec);
            self.swapout_history.push(rates.swapout_mb_per_sec);
            self.fault_history.push(rates.fault_per_sec);
            self.major_fault_history.push(rates.major_fault_per_sec);

            self.latest_rates = Some(rates);
        }

        if let Some(psi_snap) = &psi {
            self.psi_some_history.push(psi_snap.some_avg10);
            self.psi_full_history.push(psi_snap.full_avg10);
        }

        self.prev_vmstat = Some(vmstat);
        self.latest_psi = psi;
        self.latest_info = Some(info);
        Ok(())
    }
}
