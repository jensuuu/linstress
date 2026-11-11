//! Frequency and power tuning for Linux.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CPU_DIR: &str = "/sys/devices/system/cpu";

// ---------------------------------------------------------------------------
// CPU
// ---------------------------------------------------------------------------

/// Hardware frequency limits in MHz, as `(minimum, maximum)`.
pub fn cpu_frequency_limits_mhz() -> Option<(u32, u32)> {
    let read_khz = |file: &str| -> Option<u32> {
        fs::read_to_string(format!("{CPU_DIR}/cpu0/cpufreq/{file}"))
            .ok()?
            .trim()
            .parse()
            .ok()
    };
    Some((read_khz("cpuinfo_min_freq")? / 1000, read_khz("cpuinfo_max_freq")? / 1000))
}

/// The maximum frequency currently allowed by the kernel, in MHz.
pub fn current_max_mhz() -> Option<u32> {
    let khz: u32 = fs::read_to_string(format!("{CPU_DIR}/cpu0/cpufreq/scaling_max_freq"))
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(khz / 1000)
}

/// Sets the maximum frequency on every core. The value is clamped to the
/// hardware limits so a wrong number can never be written.
pub fn set_cpu_max_mhz(requested_mhz: u32) -> Result<String, String> {
    let (min_mhz, max_mhz) = cpu_frequency_limits_mhz().ok_or("cpufreq is not available")?;
    let safe_mhz = requested_mhz.clamp(min_mhz, max_mhz);

    write_to_every_core("scaling_max_freq", &(safe_mhz * 1000).to_string())?;
    Ok(format!("CPU max frequency set to {safe_mhz} MHz"))
}

/// Sets the governor on every core, e.g. "performance" or "powersave".
pub fn set_cpu_governor(governor: &str) -> Result<String, String> {
    write_to_every_core("scaling_governor", governor)?;
    Ok(format!("CPU governor set to {governor}"))
}

fn write_to_every_core(file_name: &str, value: &str) -> Result<(), String> {
    let mut written = 0;

    for entry in fs::read_dir(CPU_DIR).map_err(|e| e.to_string())?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_core = name.len() > 3
            && name.starts_with("cpu")
            && name[3..].chars().all(|c| c.is_ascii_digit());
        if !is_core {
            continue;
        }

        let path: PathBuf = entry.path().join("cpufreq").join(file_name);
        fs::write(&path, value)
            .map_err(|e| format!("Cannot write {}: {e} (run the program as root)", path.display()))?;
        written += 1;
    }

    if written == 0 {
        Err("No CPU cores with cpufreq support found".into())
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// NVIDIA GPU (through nvidia-smi)
// ---------------------------------------------------------------------------

/// What the NVIDIA driver reports about the first GPU.
#[derive(Clone, Copy, Debug)]
pub struct NvidiaInfo {
    pub power_min_w: u32,
    pub power_max_w: u32,
    pub power_now_w: u32,
    pub clock_max_mhz: u32,
}

/// Reads the power-limit range and the highest core clock of the first GPU.
/// Returns `None` when there is no NVIDIA GPU, no `nvidia-smi`, or the card
/// does not report these values (some laptop GPUs print `[N/A]`).
pub fn nvidia_info() -> Option<NvidiaInfo> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=power.min_limit,power.max_limit,power.limit,clocks.max.graphics",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let values = text
        .lines()
        .next()?
        .split(',')
        .map(|part| part.trim().parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()
        .ok()?;
    if values.len() < 4 {
        return None;
    }

    Some(NvidiaInfo {
        power_min_w: values[0].round() as u32,
        power_max_w: values[1].round() as u32,
        power_now_w: values[2].round() as u32,
        clock_max_mhz: values[3].round() as u32,
    })
}

/// Sets the GPU power limit in watts.
pub fn nvidia_set_power_limit(watts: u32) -> Result<String, String> {
    run_nvidia_smi(&["-pl", &watts.to_string()])?;
    Ok(format!("NVIDIA power limit set to {watts} W"))
}

/// Locks the GPU core clock to a range, in MHz.
pub fn nvidia_lock_clocks(min_mhz: u32, max_mhz: u32) -> Result<String, String> {
    run_nvidia_smi(&["-lgc", &format!("{min_mhz},{max_mhz}")])?;
    Ok(format!("NVIDIA clocks locked to {min_mhz}-{max_mhz} MHz"))
}

/// Removes the clock lock and returns to default behaviour.
pub fn nvidia_reset_clocks() -> Result<String, String> {
    run_nvidia_smi(&["-rgc"])?;
    Ok("NVIDIA clocks reset to default".into())
}

fn run_nvidia_smi(args: &[&str]) -> Result<(), String> {
    let output = Command::new("nvidia-smi")
        .args(args)
        .output()
        .map_err(|e| format!("Cannot run nvidia-smi: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
