//! Reads live hardware information from /proc and /sys.
//! Every function returns `Option` so the UI can show "N/A"
//! when a sensor does not exist on this machine.

use std::fs;
use std::path::Path;
use std::process::Command;

// ---------------------------------------------------------------------------
// CPU usage
// ---------------------------------------------------------------------------

/// A snapshot of the counters in the first line of /proc/stat.
struct CpuTimes {
    idle: u64,
    total: u64,
}

fn read_cpu_times() -> Option<CpuTimes> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let first_line = stat.lines().next()?; // "cpu  user nice system idle iowait ..."

    let values: Vec<u64> = first_line
        .split_whitespace()
        .skip(1) // skip the "cpu" label
        .filter_map(|v| v.parse().ok())
        .collect();

    if values.len() < 5 {
        return None;
    }

    let idle = values[3] + values[4]; // idle + iowait
    let total = values.iter().sum();
    Some(CpuTimes { idle, total })
}

/// Computes CPU usage in percent by comparing two snapshots over time.
pub struct CpuUsageMeter {
    previous: Option<CpuTimes>,
}

impl CpuUsageMeter {
    pub fn new() -> Self {
        Self { previous: read_cpu_times() }
    }

    /// Call this once per refresh. Returns usage between 0.0 and 100.0.
    pub fn sample(&mut self) -> f32 {
        let current = read_cpu_times();

        let usage = match (&self.previous, &current) {
            (Some(old), Some(new)) => {
                let total_diff = new.total.saturating_sub(old.total);
                let idle_diff = new.idle.saturating_sub(old.idle);
                if total_diff == 0 {
                    0.0
                } else {
                    100.0 * (total_diff - idle_diff) as f32 / total_diff as f32
                }
            }
            _ => 0.0,
        };

        self.previous = current;
        usage
    }
}

// ---------------------------------------------------------------------------
// CPU frequency
// ---------------------------------------------------------------------------

/// Average current frequency of all cores, in MHz.
pub fn cpu_frequency_mhz() -> Option<f32> {
    let mut sum_khz = 0.0_f64;
    let mut core_count = 0;

    for entry in fs::read_dir("/sys/devices/system/cpu").ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_cpu_dir = name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit());
        if !is_cpu_dir || name.len() == 3 {
            continue;
        }

        let freq_file = entry.path().join("cpufreq/scaling_cur_freq");
        if let Some(khz) = read_number(&freq_file) {
            sum_khz += khz;
            core_count += 1;
        }
    }

    if core_count == 0 {
        None
    } else {
        Some((sum_khz / core_count as f64 / 1000.0) as f32)
    }
}

// ---------------------------------------------------------------------------
// Temperatures
// ---------------------------------------------------------------------------

/// CPU package temperature in °C (Intel coretemp, AMD k10temp/zenpower).
pub fn cpu_temperature_celsius() -> Option<f32> {
    const CPU_SENSOR_NAMES: [&str; 4] = ["coretemp", "k10temp", "zenpower", "cpu_thermal"];

    for entry in fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let hwmon_dir = entry.path();
        let sensor_name = fs::read_to_string(hwmon_dir.join("name")).unwrap_or_default();

        if CPU_SENSOR_NAMES.contains(&sensor_name.trim()) {
            if let Some(millidegrees) = read_number(&hwmon_dir.join("temp1_input")) {
                return Some(millidegrees as f32 / 1000.0);
            }
        }
    }
    None
}

/// GPU temperature in °C. Tries AMD (hwmon) first, then NVIDIA (nvidia-smi).
pub fn gpu_temperature_celsius() -> Option<f32> {
    // AMD GPUs expose their temperature through hwmon.
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let hwmon_dir = entry.path();
            let sensor_name = fs::read_to_string(hwmon_dir.join("name")).unwrap_or_default();
            if sensor_name.trim() == "amdgpu" {
                if let Some(millidegrees) = read_number(&hwmon_dir.join("temp1_input")) {
                    return Some(millidegrees as f32 / 1000.0);
                }
            }
        }
    }

    // NVIDIA GPUs: ask the driver tool.
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;

    String::from_utf8_lossy(&output.stdout).trim().lines().next()?.parse().ok()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_number(path: &Path) -> Option<f64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// Number of logical CPU threads.
pub fn logical_cpu_count() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}

// ---------------------------------------------------------------------------
// Memory
// ---------------------------------------------------------------------------

/// Returns `(total_mb, available_mb)` of system RAM, read from /proc/meminfo.
pub fn memory_mb() -> Option<(u64, u64)> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;

    let read_kb = |label: &str| -> Option<u64> {
        let line = meminfo.lines().find(|line| line.starts_with(label))?;
        line.split_whitespace().nth(1)?.parse().ok()
    };

    Some((read_kb("MemTotal:")? / 1024, read_kb("MemAvailable:")? / 1024))
}
