use super::sysfs::{external_power_online, select_cpu_zones, thermal_state_from_zones};
use crate::models::DeviceInfoResponse;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn read(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

pub fn get_device_info() -> crate::Result<DeviceInfoResponse> {
    // hostname (native dosyalardan)
    let device_name = read("/proc/sys/kernel/hostname").or_else(|| read("/etc/hostname"));

    Ok(DeviceInfoResponse {
        device_name,
        manufacturer: read("/sys/class/dmi/id/sys_vendor"),
        model: read("/sys/class/dmi/id/product_name"),
        uuid: read("/sys/class/dmi/id/product_uuid").or_else(|| read("/etc/machine-id")), // fallback
        serial: read("/sys/class/dmi/id/product_serial"),
        android_id: None,
    })
}

pub fn get_display_refresh_rate() -> Option<f64> {
    use std::process::Command;

    // Try using xrandr, standard on X11
    let output = Command::new("xrandr")
        .args(["--current"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());

    if let Some(out) = output {
        // Output format example line:
        // "   1920x1080     60.00*+  59.96    59.93"
        // The asterisk (*) marks the current mode
        for line in out.lines() {
            if line.contains('*') {
                // Split by whitespace and find the part with '*'
                if let Some(part) = line.split_whitespace().find(|s| s.contains('*')) {
                    let clean = part.replace("*", "").replace("+", "");
                    return clean.parse::<f64>().ok();
                }
            }
        }
    }
    None
}

// ============================================================================
// Power state
// ============================================================================

/// A CPU/SoC thermal zone, resolved once: which file to poll and its critical trip.
struct CpuZone {
    temp: PathBuf,
    critical: Option<f64>,
}

/// Reads the power-saving mode, thermal state and AC power from sysfs.
///
/// - Low power: the ACPI platform profile (`/sys/firmware/acpi/platform_profile`),
///   which power-profiles-daemon and desktop power menus drive.
/// - Thermal: the CPU/SoC `/sys/class/thermal/thermal_zone*` zones, judged by
///   their headroom to their own critical trip point (see [`thermal_state_from_zones`]).
/// - External power: any online non-battery `/sys/class/power_supply/*`.
pub fn get_power_state() -> crate::models::PowerState {
    let low_power_mode = read("/sys/firmware/acpi/platform_profile").map(|p| p == "low-power");

    let zones: Vec<(f64, Option<f64>)> = cpu_zones()
        .iter()
        .filter_map(|zone| Some((read_millidegrees(&zone.temp)?, zone.critical)))
        .collect();

    crate::models::PowerState {
        low_power_mode,
        thermal_state: thermal_state_from_zones(&zones),
        on_external_power: on_external_power(),
    }
}

/// The CPU/SoC thermal zones and their critical trips. Zone types and trip
/// points are static, so they are scanned once; each poll then reads only `temp`.
fn cpu_zones() -> &'static [CpuZone] {
    static ZONES: OnceLock<Vec<CpuZone>> = OnceLock::new();
    ZONES.get_or_init(|| {
        let zones: Vec<(String, PathBuf)> = fs::read_dir("/sys/class/thermal")
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("thermal_zone"))
            .map(|e| {
                let dir = e.path();
                let kind = read_path(&dir.join("type")).unwrap_or_default();
                (kind, dir)
            })
            .collect();

        select_cpu_zones(zones)
            .into_iter()
            .map(|dir| CpuZone {
                critical: critical_trip(&dir),
                temp: dir.join("temp"),
            })
            .collect()
    })
}

/// Whether any non-battery power supply (AC adapter, USB-C charger) is online.
fn on_external_power() -> Option<bool> {
    let supplies: Vec<(String, Option<bool>)> = fs::read_dir("/sys/class/power_supply")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let dir = e.path();
            let kind = read_path(&dir.join("type"))?;
            let online = read_path(&dir.join("online")).map(|v| v == "1");
            Some((kind, online))
        })
        .collect();
    external_power_online(&supplies)
}

fn read_path(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_millidegrees(path: &Path) -> Option<f64> {
    let raw: f64 = fs::read_to_string(path).ok()?.trim().parse().ok()?;
    Some(raw / 1000.0)
}

/// Finds the zone's `critical` trip point temperature, in °C.
fn critical_trip(zone: &Path) -> Option<f64> {
    (0..16).find_map(|i| {
        let kind = fs::read_to_string(zone.join(format!("trip_point_{i}_type"))).ok()?;
        if kind.trim() != "critical" {
            return None;
        }
        read_millidegrees(&zone.join(format!("trip_point_{i}_temp")))
    })
}

/// Highest `cpuinfo_max_freq` across CPUs, in MHz. sysinfo only reports the
/// *current* clock on Linux, which under-reads an idle CPU.
pub fn get_cpu_max_frequency_mhz() -> Option<u64> {
    fs::read_dir("/sys/devices/system/cpu")
        .ok()?
        .flatten()
        .filter_map(|e| {
            let khz: u64 = fs::read_to_string(e.path().join("cpufreq/cpuinfo_max_freq"))
                .ok()?
                .trim()
                .parse()
                .ok()?;
            Some(khz / 1000)
        })
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_returns_none_for_nonexistent_file() {
        let result = read("/nonexistent/path/to/file");
        assert!(result.is_none());
    }

    #[test]
    fn read_trims_whitespace() {
        // Create a temp file with whitespace
        use std::io::Write;
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_read_trim.txt");

        {
            let mut file = std::fs::File::create(&temp_file).unwrap();
            writeln!(file, "  test value  ").unwrap();
        }

        let result = read(temp_file.to_str().unwrap());
        assert_eq!(result, Some("test value".to_string()));

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn parse_xrandr_refresh_rate() {
        // Simulated xrandr output line
        let xrandr_line = "   1920x1080     60.00*+  59.96    59.93";

        // Extract refresh rate like our function does
        if let Some(part) = xrandr_line.split_whitespace().find(|s| s.contains('*')) {
            let clean = part.replace("*", "").replace("+", "");
            let rate: f64 = clean.parse().unwrap();
            assert!((rate - 60.0).abs() < 0.01);
        } else {
            panic!("Failed to find refresh rate in xrandr line");
        }
    }

    #[test]
    fn parse_xrandr_without_plus() {
        // Some systems show just asterisk without plus
        let xrandr_line = "   2560x1440     144.00*  120.00    60.00";

        if let Some(part) = xrandr_line.split_whitespace().find(|s| s.contains('*')) {
            let clean = part.replace("*", "").replace("+", "");
            let rate: f64 = clean.parse().unwrap();
            assert!((rate - 144.0).abs() < 0.01);
        } else {
            panic!("Failed to find refresh rate");
        }
    }
}
