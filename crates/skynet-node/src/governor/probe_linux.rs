//! `probe_linux`: Fast rootless Linux sensor probe reading `/proc`, `/sys/class/thermal`,
//! `/sys/class/power_supply`, `/sys/class/drm`, and optional `nvidia-smi`.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

use skynet_agent::SystemTelemetrySnapshot;

use crate::governor::matches_game_watchlist;
use crate::governor::probe_windows::parse_nvidia_smi_csv;

/// Rootless Linux telemetry probe backed by `/proc` and `/sys` virtual filesystems.
#[derive(Debug, Default)]
pub struct LinuxTelemetryProbe {
    custom_watchlist: Vec<String>,
    prev_cpu_jiffies: Mutex<Option<(u64, u64)>>,
}

impl LinuxTelemetryProbe {
    pub fn new(custom_watchlist: Vec<String>) -> Self {
        Self { custom_watchlist, prev_cpu_jiffies: Mutex::new(None) }
    }

    pub fn sample(&self, observed_at_sec: u64) -> SystemTelemetrySnapshot {
        let cpu_util_pct = self.sample_cpu_pct();
        let ram_used_pct = fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| parse_meminfo_used_pct(&s))
            .unwrap_or(15);
        let sys_temp = sample_sysfs_thermal_c();
        let (drm_gpu, drm_vram) = sample_drm_gpu_metrics();
        let (gpu_util_pct, vram_used_pct, temp_c) =
            merge_nvidia_if_present(drm_gpu, drm_vram.unwrap_or(ram_used_pct), sys_temp);

        SystemTelemetrySnapshot {
            gpu_util_pct,
            cpu_util_pct,
            vram_used_pct,
            temp_c,
            on_battery: sample_sysfs_on_battery(),
            active_game_detected: scan_proc_comm_watchlist(&self.custom_watchlist),
            observed_at_sec,
        }
    }

    fn sample_cpu_pct(&self) -> u8 {
        let Some(stat) = fs::read_to_string("/proc/stat").ok() else {
            return 5;
        };
        let Some(curr) = parse_proc_stat_jiffies(&stat) else {
            return 5;
        };
        let Ok(mut guard) = self.prev_cpu_jiffies.lock() else {
            return 5;
        };
        let pct = guard.map(|prev| compute_cpu_delta_pct(prev, curr)).unwrap_or(5);
        *guard = Some(curr);
        pct
    }
}

/// Parses `(total_jiffies, idle_jiffies)` from the aggregate `cpu ` line in `/proc/stat`.
pub fn parse_proc_stat_jiffies(stat_text: &str) -> Option<(u64, u64)> {
    let line = stat_text.lines().find(|l| l.starts_with("cpu "))?;
    let vals: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|t| t.parse::<u64>().ok())
        .collect();
    if vals.len() < 4 {
        return None;
    }
    let total: u64 = vals.iter().sum();
    let idle = vals[3].saturating_add(vals.get(4).copied().unwrap_or(0));
    Some((total, idle))
}

/// Computes CPU busy percentage from two `(total, idle)` `/proc/stat` snapshots.
pub fn compute_cpu_delta_pct(prev: (u64, u64), curr: (u64, u64)) -> u8 {
    let dt = curr.0.saturating_sub(prev.0);
    if dt == 0 {
        return 0;
    }
    let didle = curr.1.saturating_sub(prev.1).min(dt);
    (((dt - didle).saturating_mul(100)) / dt).min(100) as u8
}

/// Parses memory usage percentage from `/proc/meminfo`.
pub fn parse_meminfo_used_pct(meminfo: &str) -> Option<u8> {
    let mut total_kb = None;
    let mut avail_kb = None;
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = line.split_whitespace().nth(1).and_then(|v| v.parse::<u64>().ok());
        } else if line.starts_with("MemAvailable:") {
            avail_kb = line.split_whitespace().nth(1).and_then(|v| v.parse::<u64>().ok());
        }
    }
    let total = total_kb.filter(|&t| t > 0)?;
    let used = total.saturating_sub(avail_kb.unwrap_or(total / 2));
    Some(((used.saturating_mul(100)) / total).min(100) as u8)
}

/// Converts sysfs thermal zone millidegrees (`78000`) or degrees (`78`) into `u8` °C.
pub fn parse_sysfs_millideg_c(raw: &str) -> Option<u8> {
    let val = raw.trim().parse::<i64>().ok()?;
    if val <= 0 {
        return None;
    }
    let celsius = if val >= 1_000 { val / 1_000 } else { val };
    Some(celsius.clamp(0, 120) as u8)
}

fn sample_sysfs_thermal_c() -> u8 {
    let Ok(entries) = fs::read_dir("/sys/class/thermal") else {
        return 45;
    };
    entries
        .flatten()
        .filter_map(|e| fs::read_to_string(e.path().join("temp")).ok())
        .filter_map(|s| parse_sysfs_millideg_c(&s))
        .max()
        .unwrap_or(45)
}

fn sample_sysfs_on_battery() -> bool {
    let Ok(entries) = fs::read_dir("/sys/class/power_supply") else {
        return false;
    };
    for entry in entries.flatten() {
        let status = fs::read_to_string(entry.path().join("status")).unwrap_or_default();
        if status.trim().eq_ignore_ascii_case("Discharging") {
            return true;
        }
    }
    false
}

fn sample_drm_gpu_metrics() -> (u8, Option<u8>) {
    let dev = Path::new("/sys/class/drm/card0/device");
    let gpu = fs::read_to_string(dev.join("gpu_busy_percent"))
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0)
        .min(100);
    let used = fs::read_to_string(dev.join("mem_info_vram_used"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());
    let total = fs::read_to_string(dev.join("mem_info_vram_total"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|&t| t > 0);
    let vram_pct = used.zip(total).map(|(u, t)| ((u.saturating_mul(100)) / t).min(100) as u8);
    (gpu, vram_pct)
}

fn merge_nvidia_if_present(drm_gpu: u8, fallback_vram: u8, sys_temp: u8) -> (u8, u8, u8) {
    if !Path::new("/dev/nvidia0").exists() {
        return (drm_gpu, fallback_vram, sys_temp);
    }
    let out = Command::new("nvidia-smi")
        .args(["--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu", "--format=csv,noheader,nounits"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    parse_nvidia_smi_csv(&out).unwrap_or((drm_gpu, fallback_vram, sys_temp))
}

fn scan_proc_comm_watchlist(custom: &[String]) -> bool {
    let Ok(entries) = fs::read_dir("/proc") else {
        return false;
    };
    for entry in entries.flatten() {
        let is_pid = entry
            .file_name()
            .to_str()
            .map(|s| s.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false);
        if !is_pid {
            continue;
        }
        if let Ok(comm) = fs::read_to_string(entry.path().join("comm")) {
            if matches_game_watchlist(&comm, custom) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_proc_stat_meminfo_and_sysfs_thermal_parsers() {
        let p1 = parse_proc_stat_jiffies("cpu  100 0 50 850 0 0 0 0\n").unwrap();
        let p2 = parse_proc_stat_jiffies("cpu  160 0 90 950 0 0 0 0\n").unwrap();
        assert_eq!(compute_cpu_delta_pct(p1, p2), 50);

        let mem = "MemTotal:       16000000 kB\nMemAvailable:   12000000 kB\n";
        assert_eq!(parse_meminfo_used_pct(mem), Some(25));
        assert_eq!(parse_sysfs_millideg_c("78500\n"), Some(78));
    }
}
