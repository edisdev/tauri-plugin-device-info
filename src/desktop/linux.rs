use crate::models::DeviceInfoResponse;
use std::fs;

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

// ============================================================================
// Power state
// ============================================================================

/// Critical trip point assumed for a thermal zone that doesn't report one.
const DEFAULT_CRITICAL_C: f64 = 100.0;

/// Reads the power-saving mode and thermal state from sysfs.
///
/// - Low power: the ACPI platform profile (`/sys/firmware/acpi/platform_profile`),
///   which power-profiles-daemon and desktop power menus drive.
/// - Thermal: every `/sys/class/thermal/thermal_zone*`, judged by its headroom
///   to its own critical trip point (see [`thermal_state_from_zones`]).
pub fn get_power_state() -> crate::models::PowerState {
    let low_power_mode = read("/sys/firmware/acpi/platform_profile").map(|p| p == "low-power");

    let zones: Vec<(f64, Option<f64>)> = fs::read_dir("/sys/class/thermal")
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("thermal_zone"))
        .filter_map(|e| {
            let dir = e.path();
            let temp = read_millidegrees(&dir.join("temp"))?;
            Some((temp, critical_trip(&dir)))
        })
        .collect();

    crate::models::PowerState {
        low_power_mode,
        thermal_state: thermal_state_from_zones(&zones),
    }
}

fn read_millidegrees(path: &std::path::Path) -> Option<f64> {
    let raw: f64 = fs::read_to_string(path).ok()?.trim().parse().ok()?;
    Some(raw / 1000.0)
}

/// Finds the zone's `critical` trip point temperature, in °C.
fn critical_trip(zone: &std::path::Path) -> Option<f64> {
    (0..16).find_map(|i| {
        let kind = fs::read_to_string(zone.join(format!("trip_point_{i}_type"))).ok()?;
        if kind.trim() != "critical" {
            return None;
        }
        read_millidegrees(&zone.join(format!("trip_point_{i}_temp")))
    })
}

/// Maps `(temperature °C, critical trip °C)` per zone to the hottest zone's state.
///
/// Headroom to critical: ≤5 °C → critical, ≤15 → serious, ≤25 → fair. Readings
/// outside a plausible range (sensors reporting 0, -273, or garbage) are
/// ignored, as are implausible trip points.
pub(crate) fn thermal_state_from_zones(
    zones: &[(f64, Option<f64>)],
) -> Option<crate::models::ThermalState> {
    use crate::models::ThermalState;

    zones
        .iter()
        .filter(|(temp, _)| (1.0..=150.0).contains(temp))
        .map(|(temp, crit)| {
            let crit = crit
                .filter(|c| (40.0..=150.0).contains(c))
                .unwrap_or(DEFAULT_CRITICAL_C);
            match crit - temp {
                h if h <= 5.0 => ThermalState::Critical,
                h if h <= 15.0 => ThermalState::Serious,
                h if h <= 25.0 => ThermalState::Fair,
                _ => ThermalState::Nominal,
            }
        })
        .max()
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
    use crate::models::ThermalState;

    #[test]
    fn no_zones_means_unknown() {
        assert_eq!(thermal_state_from_zones(&[]), None);
    }

    #[test]
    fn headroom_to_critical_trip_sets_state() {
        assert_eq!(
            thermal_state_from_zones(&[(50.0, Some(100.0))]),
            Some(ThermalState::Nominal)
        );
        assert_eq!(
            thermal_state_from_zones(&[(76.0, Some(100.0))]),
            Some(ThermalState::Fair)
        );
        assert_eq!(
            thermal_state_from_zones(&[(86.0, Some(100.0))]),
            Some(ThermalState::Serious)
        );
        assert_eq!(
            thermal_state_from_zones(&[(96.0, Some(100.0))]),
            Some(ThermalState::Critical)
        );
        // A lower critical trip makes the same temperature more severe.
        assert_eq!(
            thermal_state_from_zones(&[(76.0, Some(85.0))]),
            Some(ThermalState::Serious)
        );
    }

    #[test]
    fn hottest_zone_wins_and_garbage_is_ignored() {
        let zones = [
            (40.0, Some(105.0)),
            (88.0, None),          // default critical 100 → serious
            (-273.0, Some(100.0)), // bogus sensor
            (500.0, Some(100.0)),  // bogus sensor
            (60.0, Some(0.0)),     // bogus trip → default 100 → nominal
        ];
        assert_eq!(
            thermal_state_from_zones(&zones),
            Some(ThermalState::Serious)
        );
    }
}
