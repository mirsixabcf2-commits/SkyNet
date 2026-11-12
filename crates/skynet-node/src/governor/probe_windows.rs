//! `probe_windows`: Rootless Windows hardware sensor probe (`nvidia-smi` CSV,
//! `tasklist /FO CSV /NH`, WMIC battery) and deterministic `SimulatedTelemetryProbe`.

use std::process::Command;
use std::sync::{Arc, RwLock};

use skynet_agent::SystemTelemetrySnapshot;
use tokio::sync::Notify;

use crate::governor::matches_game_watchlist;

/// Rootless Windows telemetry collector using standard inbox CLI utilities and `nvidia-smi`.
#[derive(Debug, Clone, Default)]
pub struct WindowsTelemetryProbe {
    custom_watchlist: Vec<String>,
}

impl WindowsTelemetryProbe {
    pub fn new(custom_watchlist: Vec<String>) -> Self {
        Self { custom_watchlist }
    }

    pub fn sample(&self, observed_at_sec: u64) -> SystemTelemetrySnapshot {
        let smi_out = run_win_cmd(
            "nvidia-smi",
            &[
                "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu",
                "--format=csv,noheader,nounits",
            ],
        );
        let (gpu_util_pct, vram_used_pct, temp_c) =
            parse_nvidia_smi_csv(&smi_out).unwrap_or((5, 15, 45));

        let tasks_out = run_win_cmd("tasklist", &["/FO", "CSV", "/NH"]);
        let active_game_detected =
            parse_tasklist_csv_for_game(&tasks_out, &self.custom_watchlist);

        let batt_out = run_win_cmd("WMIC", &["Path", "Win32_Battery", "Get", "BatteryStatus"]);
        let on_battery = parse_wmic_battery_status(&batt_out);

        SystemTelemetrySnapshot {
            gpu_util_pct,
            cpu_util_pct: gpu_util_pct / 2,
            vram_used_pct,
            temp_c,
            on_battery,
            active_game_detected,
            observed_at_sec,
        }
    }
}

fn run_win_cmd(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// Parses `nvidia-smi` CSV line `util_gpu, mem_used, mem_total, temp_c` into `(gpu%, vram%, °C)`.
pub fn parse_nvidia_smi_csv(csv_text: &str) -> Option<(u8, u8, u8)> {
    let line = csv_text.lines().find(|l| !l.trim().is_empty())?;
    let cols: Vec<&str> = line.split(',').map(str::trim).collect();
    if cols.len() < 4 {
        return None;
    }
    let gpu = cols[0].parse::<u64>().ok()?.min(100) as u8;
    let mem_used = cols[1].parse::<u64>().ok()?;
    let mem_total = cols[2].parse::<u64>().ok().filter(|&t| t > 0)?;
    let temp = cols[3].parse::<u64>().ok()?.min(120) as u8;
    let vram = ((mem_used.saturating_mul(100)) / mem_total).min(100) as u8;
    Some((gpu, vram, temp))
}

/// Checks `tasklist /FO CSV /NH` image names against default and custom game watchlists.
pub fn parse_tasklist_csv_for_game(csv_lines: &str, custom: &[String]) -> bool {
    for line in csv_lines.lines() {
        let first_col = line.split(',').next().unwrap_or("").trim_matches('"');
        if !first_col.is_empty() && matches_game_watchlist(first_col, custom) {
            return true;
        }
    }
    false
}

/// Parses WMIC `Win32_Battery.BatteryStatus` (`1` = Discharging on battery, `2` = AC power).
pub fn parse_wmic_battery_status(wmic_out: &str) -> bool {
    wmic_out
        .lines()
        .filter_map(|l| l.trim().parse::<u8>().ok())
        .any(|code| code == 1)
}

/// Deterministic, lock-synchronized cross-platform telemetry injector for simulations and tests.
#[derive(Debug, Clone)]
pub struct SimulatedTelemetryProbe {
    snapshot: Arc<RwLock<SystemTelemetrySnapshot>>,
    wake: Arc<Notify>,
}

impl Default for SimulatedTelemetryProbe {
    fn default() -> Self {
        Self::new(SystemTelemetrySnapshot::default())
    }
}

impl SimulatedTelemetryProbe {
    pub fn new(initial: SystemTelemetrySnapshot) -> Self {
        Self {
            snapshot: Arc::new(RwLock::new(initial)),
            wake: Arc::new(Notify::new()),
        }
    }

    pub fn set_snapshot(&self, next: SystemTelemetrySnapshot) {
        if let Ok(mut guard) = self.snapshot.write() {
            *guard = next;
        }
        self.wake.notify_waiters();
    }

    pub fn update<F: FnOnce(&mut SystemTelemetrySnapshot)>(&self, mutate: F) {
        if let Ok(mut guard) = self.snapshot.write() {
            mutate(&mut guard);
        }
        self.wake.notify_waiters();
    }

    pub fn sample(&self) -> SystemTelemetrySnapshot {
        self.snapshot.read().map(|g| *g).unwrap_or_default()
    }

    pub fn wake_notifier(&self) -> Arc<Notify> {
        Arc::clone(&self.wake)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_parsers_and_simulated_probe() {
        assert_eq!(parse_nvidia_smi_csv("35, 4096, 16384, 68\n"), Some((35, 25, 68)));
        let tasklist = "\"explorer.exe\",\"1200\",\"Console\",\"1\",\"80,000 K\"\n\"cs2.exe\",\"9400\",\"Console\",\"1\",\"4,200,000 K\"";
        assert!(parse_tasklist_csv_for_game(tasklist, &[]));
        assert!(parse_wmic_battery_status("BatteryStatus\n1\n"));
        assert!(!parse_wmic_battery_status("BatteryStatus\n2\n"));

        let sim = SimulatedTelemetryProbe::default();
        sim.update(|s| s.temp_c = 79);
        assert_eq!(sim.sample().temp_c, 79);
    }
}
