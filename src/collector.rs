use std::fs;
use anyhow::Result;

/// Snapshot of memory usage parsed from /proc/meminfo.
#[derive(Debug, Clone)]
pub struct MemInfo {
    pub ram_total_kb: u64,
    pub ram_used_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
}

impl MemInfo {
    pub fn ram_pct(&self) -> f64 {
        if self.ram_total_kb == 0 {
            return 0.0;
        }
        (self.ram_used_kb as f64 / self.ram_total_kb as f64) * 100.0
    }

    pub fn swap_pct(&self) -> f64 {
        if self.swap_total_kb == 0 {
            return 0.0;
        }
        (self.swap_used_kb as f64 / self.swap_total_kb as f64) * 100.0
    }
}

pub fn read_meminfo() -> Result<MemInfo> {
    let content = fs::read_to_string("/proc/meminfo")?;

    let mut mem_total: u64 = 0;
    let mut mem_available: u64 = 0;
    let mut swap_total: u64 = 0;
    let mut swap_free: u64 = 0;

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let key = parts.next().unwrap_or("");
        let value: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);

        match key {
            "MemTotal:" => mem_total = value,
            "MemAvailable:" => mem_available = value,
            "SwapTotal:" => swap_total = value,
            "SwapFree:" => swap_free = value,
            _ => {}
        }
    }

    Ok(MemInfo {
        ram_total_kb: mem_total,
        ram_used_kb: mem_total.saturating_sub(mem_available),
        swap_total_kb: swap_total,
        swap_used_kb: swap_total.saturating_sub(swap_free),
    })
}

fn human_bytes(kb: u64) -> String {
    let gib = kb as f64 / (1024.0 * 1024.0);
    if gib >= 1.0 {
        format!("{:.1}GiB", gib)
    } else {
        let mib = kb as f64 / 1024.0;
        format!("{:.0}MiB", mib)
    }
}

impl MemInfo {
    pub fn ram_label(&self) -> String {
        format!(
            "RAM: {:>3.0}%    {}/{}",
            self.ram_pct(),
            human_bytes(self.ram_used_kb),
            human_bytes(self.ram_total_kb),
        )
    }

    pub fn swap_label(&self) -> String {
        format!(
            "SWP: {:>3.0}%    {}/{}",
            self.swap_pct(),
            human_bytes(self.swap_used_kb),
            human_bytes(self.swap_total_kb),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentages() {
        let info = MemInfo {
            ram_total_kb: 1000,
            ram_used_kb: 150,
            swap_total_kb: 500,
            swap_used_kb: 50,
        };
        assert!((info.ram_pct() - 15.0).abs() < 0.01);
        assert!((info.swap_pct() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_zero_total() {
        let info = MemInfo {
            ram_total_kb: 0,
            ram_used_kb: 0,
            swap_total_kb: 0,
            swap_used_kb: 0,
        };
        assert_eq!(info.ram_pct(), 0.0);
        assert_eq!(info.swap_pct(), 0.0);
    }

    #[test]
    fn test_human_bytes() {
        assert_eq!(human_bytes(512), "0MiB");
        assert_eq!(human_bytes(1024), "1MiB");
        assert_eq!(human_bytes(1048576), "1.0GiB");
        assert_eq!(human_bytes(65798144), "62.8GiB");
    }

    #[test]
    fn test_labels() {
        let info = MemInfo {
            ram_total_kb: 65798144,
            ram_used_kb: 9961472,
            swap_total_kb: 8388608,
            swap_used_kb: 0,
        };
        assert!(info.ram_label().contains("RAM:"));
        assert!(info.swap_label().contains("SWP:"));
    }
}
