//! `probe_macos`: Rootless macOS hardware telemetry & active game sensor probe
//! using `ioreg`, `pmset`, `sysctl`, `lsappinfo`, and `ps` (`0` external crates).

use std::process::Command;

use skynet_agent::SystemTelemetrySnapshot;

use crate::governor::matches_game_watchlist;

const BASELINE_MAC_TEMP_C: u8 = 44;

/// Rootless macOS sensor collector for Apple Silicon and Intel Macs.
#[derive(Debug, Clone, Default)]
pub struct MacOsTelemetryProbe {
    custom_watchlist: Vec<String>,
}

impl MacOsTelemetryProbe {
    pub fn new(custom_watchlist: Vec<String>) -> Self {
        Self { custom_watchlist }
    }

    pub fn sample(&self, observed_at_sec: u64) -> SystemTelemetrySnapshot {
        let ioreg_out = run_cmd("ioreg", &["-r", "-d", "1", "-w", "0", "-c", "IOAccelerator"]);
        let (gpu_util_pct, vram_used_pct) = parse_ioreg_gpu_metrics(&ioreg_out);

        let batt_out = run_cmd("pmset", &["-g", "batt"]);
        let on_battery = parse_pmset_battery(&batt_out);

        let therm_out = run_cmd("pmset", &["-g", "therm"]);
        let sysctl_therm = run_cmd("sysctl", &["-n", "machdep.xcpm.cpu_thermal_level"]);
        let temp_c = parse_macos_thermal_c(&therm_out, &sysctl_therm, gpu_util_pct);

        let ps_cpu_out = run_cmd("ps", &["-A", "-o", "%cpu="]);
        let ncpu = run_cmd("sysctl", &["-n", "hw.ncpu"])
            .trim()
            .parse::<u32>()
            .unwrap_or(4)
            .max(1);
        let cpu_util_pct = parse_ps_cpu_pct(&ps_cpu_out, ncpu);

        let front_app = run_cmd("lsappinfo", &["front"]);
        let ps_comm = run_cmd("ps", &["-axo", "comm="]);
        let active_game_detected =
            detect_macos_game_active(&front_app, &ps_comm, &self.custom_watchlist);

        SystemTelemetrySnapshot {
            gpu_util_pct,
            cpu_util_pct,
            vram_used_pct,
            temp_c,
            on_battery,
            active_game_detected,
            observed_at_sec,
        }
    }
}

fn run_cmd(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// Parses GPU utilization `%` and unified/VRAM memory usage `%` from `ioreg -c IOAccelerator`.
pub fn parse_ioreg_gpu_metrics(ioreg_text: &str) -> (u8, u8) {
    let gpu = extract_ioreg_u64(ioreg_text, "\"Device Utilization %\"=")
        .or_else(|| extract_ioreg_u64(ioreg_text, "\"Renderer Utilization %\"="))
        .unwrap_or(0)
        .min(100) as u8;

    let in_use = extract_ioreg_u64(ioreg_text, "\"In use system memory\"=").unwrap_or(0);
    let alloc = extract_ioreg_u64(ioreg_text, "\"Alloc system memory\"=").unwrap_or(0);
    let vram_pct = in_use
        .saturating_mul(100)
        .checked_div(alloc)
        .map_or(15, |pct| pct.min(100) as u8);
    (gpu, vram_pct)
}

fn extract_ioreg_u64(text: &str, key: &str) -> Option<u64> {
    let idx = text.find(key)?;
    let rest = &text[idx + key.len()..];
    let digits: String = rest
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// Parses AC vs Battery status from `pmset -g batt`.
pub fn parse_pmset_battery(pmset_batt: &str) -> bool {
    pmset_batt.contains("Battery Power")
}

/// Estimates Celsius thermal state from rootless `pmset -g therm` & `sysctl` levels.
pub fn parse_macos_thermal_c(pmset_therm: &str, sysctl_level: &str, gpu_util_pct: u8) -> u8 {
    if let Ok(level) = sysctl_level.trim().parse::<u16>() {
        return (u16::from(BASELINE_MAC_TEMP_C) + level / 2).min(100) as u8;
    }
    if let Some(speed_limit) = extract_ioreg_u64(pmset_therm, "CPU_Speed_Limit =") {
        if speed_limit < 100 {
            let penalty = (100u64.saturating_sub(speed_limit)).min(35) as u8;
            return BASELINE_MAC_TEMP_C.saturating_add(30).saturating_add(penalty / 2);
        }
    }
    BASELINE_MAC_TEMP_C.saturating_add(gpu_util_pct / 4)
}

/// Aggregates `ps -A -o %cpu=` normalized by logical core count `ncpu`.
pub fn parse_ps_cpu_pct(ps_cpu: &str, ncpu: u32) -> u8 {
    let sum_pct: f64 = ps_cpu
        .lines()
        .filter_map(|l| l.trim().replace(',', ".").parse::<f64>().ok())
        .sum();
    ((sum_pct / f64::from(ncpu.max(1))).round().clamp(0.0, 100.0)) as u8
}

/// Matches foreground bundle/process list against default and custom game watchlists.
pub fn detect_macos_game_active(front_app: &str, ps_comm: &str, custom: &[String]) -> bool {
    matches_game_watchlist(front_app, custom) || matches_game_watchlist(ps_comm, custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ioreg_pmset_thermal_and_game_watchlist() {
        let ioreg = r#""PerformanceStatistics" = {"Device Utilization %"=62,"In use system memory"=2000,"Alloc system memory"=8000}"#;
        assert_eq!(parse_ioreg_gpu_metrics(ioreg), (62, 25));

        assert!(parse_pmset_battery("Now drawing from 'Battery Power'\n -InternalBattery-0 82%"));
        assert!(!parse_pmset_battery("Now drawing from 'AC Power'"));

        assert_eq!(parse_macos_thermal_c("CPU_Speed_Limit = 70", "", 20), 89);
        assert_eq!(parse_ps_cpu_pct("40.0\n40.0\n", 4), 20);

        let ps = "/Applications/Cyberpunk2077.app/Contents/MacOS/Cyberpunk2077\n/sbin/launchd";
        assert!(detect_macos_game_active("", ps, &[]));
        assert!(detect_macos_game_active("my_custom_sim", "", &["custom_sim".into()]));
    }
}
