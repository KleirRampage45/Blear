//! CPU usage measurement for usage stats.
//!
//! On Linux, reads /proc/self/stat to get process CPU time and total system
//! CPU time, then computes a percentage since the last sample.
//! On Windows, uses GetProcessTimes (via windows-sys).
//! On macOS, returns 0.0 (not implemented).

use std::time::Instant;

pub struct CpuSampler {
    last_sample: Option<Instant>,
    last_process_us: u64,
    last_total_us: u64,
}

impl CpuSampler {
    pub fn new() -> Self {
        Self {
            last_sample: None,
            last_process_us: 0,
            last_total_us: 0,
        }
    }

    /// Sample current CPU usage. Returns percentage (0.0-100.0) since last
    /// call. Returns 0.0 on the first call.
    pub fn sample(&mut self) -> f64 {
        let (process_us, total_us) = read_cpu_times();
        let now = Instant::now();

        let result = if let Some(last) = self.last_sample {
            let dt = now.duration_since(last).as_micros() as f64;
            if dt > 0.0 {
                let d_proc = (process_us.saturating_sub(self.last_process_us)) as f64;
                let d_total = (total_us.saturating_sub(self.last_total_us)) as f64;
                if d_total > 0.0 {
                    (d_proc / d_total) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        self.last_sample = Some(now);
        self.last_process_us = process_us;
        self.last_total_us = total_us;
        result
    }
}

#[cfg(target_os = "linux")]
fn read_cpu_times() -> (u64, u64) {
    let process_us = std::fs::read_to_string("/proc/self/stat")
        .ok()
        .and_then(|s| {
            let fields: Vec<&str> = s.split_whitespace().collect();
            // utime (field 14, index 13) + stime (field 15, index 14)
            let utime: u64 = fields.get(13)?.parse().ok()?;
            let stime: u64 = fields.get(14)?.parse().ok()?;
            // Convert from clock ticks to microseconds (typical 100 ticks/sec)
            let ticks_per_sec = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as u64;
            if ticks_per_sec == 0 {
                return Some(utime + stime);
            }
            Some((utime + stime) * 1_000_000 / ticks_per_sec)
        })
        .unwrap_or(0);

    let total_us = std::fs::read_to_string("/proc/stat")
        .ok()
        .and_then(|s| {
            // First line: cpu  user nice system idle iowait irq softirq steal guest guest_nice
            let line = s.lines().next()?;
            let total_ticks: u64 = line
                .split_whitespace()
                .skip(1)
                .filter_map(|v| v.parse::<u64>().ok())
                .sum();
            let ticks_per_sec = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as u64;
            if ticks_per_sec == 0 {
                return Some(total_ticks);
            }
            Some(total_ticks * 1_000_000 / ticks_per_sec)
        })
        .unwrap_or(0);

    (process_us, total_us)
}

#[cfg(target_os = "windows")]
fn read_cpu_times() -> (u64, u64) {
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, GetProcessTimes, GetSystemTimes,
    };

    unsafe {
        let mut process_times = std::mem::zeroed();
        let process = GetCurrentProcess();
        if GetProcessTimes(process, &mut process_times.0, &mut process_times.1, &mut process_times.2, &mut process_times.3) == 0 {
            return (0, 0);
        }
        let process_us = filetime_to_u64(&process_times.1) + filetime_to_u64(&process_times.2);

        let mut idle_ft = std::mem::zeroed();
        let mut kernel_ft = std::mem::zeroed();
        let mut user_ft = std::mem::zeroed();
        if GetSystemTimes(&mut idle_ft, &mut kernel_ft, &mut user_ft) == 0 {
            return (process_us, 0);
        }
        let total_us = filetime_to_u64(&idle_ft) + filetime_to_u64(&kernel_ft) + filetime_to_u64(&user_ft);
        (process_us, total_us)
    }
}

#[cfg(target_os = "windows")]
fn filetime_to_u64(ft: &windows_sys::Win32::Foundation::FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64) / 10
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn read_cpu_times() -> (u64, u64) {
    (0, 0)
}
