use anyhow::Result;

use crate::collector::{
    self, HardwareInfo, MemInfo, PsiSnapshot, VmRates, VmStatSnapshot,
};
use crate::ring_buffer::RingBuffer;

pub struct App {
    // RAM row
    pub alloc_history: RingBuffer,
    pub free_history: RingBuffer,
    // Swap row
    pub swapin_history: RingBuffer,
    pub swapout_history: RingBuffer,
    // Page faults row
    pub fault_history: RingBuffer,
    pub major_fault_history: RingBuffer,
    // PSI row
    pub psi_some_history: RingBuffer,
    pub psi_full_history: RingBuffer,

    pub latest_info: Option<MemInfo>,
    pub latest_rates: Option<VmRates>,
    pub latest_psi: Option<PsiSnapshot>,
    pub hardware: HardwareInfo,
    pub should_quit: bool,
    pub scrollback_secs: u64,
    refresh_ms: u64,
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
            refresh_ms,
            prev_vmstat: None,
        }
    }

    pub fn chart_capacity(&self) -> usize {
        self.alloc_history.capacity()
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
