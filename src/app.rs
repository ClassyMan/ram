use anyhow::Result;

use crate::collector::{self, MemInfo};
use crate::ring_buffer::RingBuffer;

pub struct App {
    pub ram_history: RingBuffer,
    pub swap_history: RingBuffer,
    pub latest_info: Option<MemInfo>,
    pub should_quit: bool,
    pub scrollback_secs: u64,
}

impl App {
    pub fn new(refresh_ms: u64, scrollback_secs: u64) -> Self {
        let capacity = ((scrollback_secs * 1000) / refresh_ms) as usize;
        Self {
            ram_history: RingBuffer::new(capacity),
            swap_history: RingBuffer::new(capacity),
            latest_info: None,
            should_quit: false,
            scrollback_secs,
        }
    }

    pub fn ram_history_capacity(&self) -> usize {
        self.ram_history.capacity()
    }

    pub fn tick(&mut self) -> Result<()> {
        let info = collector::read_meminfo()?;
        self.ram_history.push(info.ram_pct());
        self.swap_history.push(info.swap_pct());
        self.latest_info = Some(info);
        Ok(())
    }
}

