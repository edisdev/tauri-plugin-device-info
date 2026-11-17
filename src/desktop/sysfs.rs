//! Pure interpretation of Linux sysfs power/thermal readings.
//!
//! Kept free of file I/O and platform APIs so the rules compile and are
//! unit-tested on every host, not only on Linux; `linux.rs` does the reading.

use crate::models::ThermalState;

/// Critical trip point assumed for a thermal zone that doesn't report a plausible one.
pub(crate) const DEFAULT_CRITICAL_C: f64 = 100.0;

/// Critical trip points below this are not CPU/SoC limits (battery, skin and
/// Wi-Fi zones trip at 45-70 °C) and are replaced by [`DEFAULT_CRITICAL_C`].
pub(crate) const MIN_CPU_CRITICAL_C: f64 = 80.0;

/// Whether a thermal zone's `type` names a CPU / SoC sensor.
///
/// Covers the common drivers: Intel (`x86_pkg_temp`, `coretemp`, `TCPU`), AMD
/// (`k10temp`, `zenpower`), the ACPI CPU zone (`acpitz`) and ARM / SoC zones
/// (`cpu-thermal`, `cpu0-thermal`, `cpuss-0`, `soc_thermal`, `SoC`, `mtktscpu`).
/// Battery, skin, charger, Wi-Fi, modem and GPU zones are excluded.
pub(crate) fn is_cpu_zone_type(kind: &str) -> bool {
    let kind = kind.trim().to_ascii_lowercase();
    matches!(
        kind.as_str(),
        "x86_pkg_temp" | "coretemp" | "k10temp" | "zenpower" | "acpitz"
    ) || kind.contains("cpu")
        || kind.starts_with("soc")
}

/// Keeps the zones whose `type` is a CPU / SoC sensor (see [`is_cpu_zone_type`]).
///
/// If none match — an unusual board with vendor-specific names — every zone is
/// kept, since [`thermal_state_from_zones`] also discounts low trip points.
pub(crate) fn select_cpu_zones<T>(zones: Vec<(String, T)>) -> Vec<T> {
    if zones.iter().any(|(kind, _)| is_cpu_zone_type(kind)) {
        zones
            .into_iter()
            .filter(|(kind, _)| is_cpu_zone_type(kind))
            .map(|(_, zone)| zone)
            .collect()
    } else {
        zones.into_iter().map(|(_, zone)| zone).collect()
    }
}

/// Maps `(temperature °C, critical trip °C)` per zone to the hottest zone's state.
///
/// Headroom to critical: ≤5 °C → critical, ≤15 → serious, ≤25 → fair. Readings
/// outside a plausible range (sensors reporting 0, -273, or garbage) are
/// ignored; trip points outside 80-150 °C are replaced by the 100 °C default.
pub(crate) fn thermal_state_from_zones(zones: &[(f64, Option<f64>)]) -> Option<ThermalState> {
    zones
        .iter()
        .filter(|(temp, _)| (1.0..=150.0).contains(temp))
        .map(|(temp, crit)| {
            let crit = crit
                .filter(|c| (MIN_CPU_CRITICAL_C..=150.0).contains(c))
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

/// Whether the machine is on external power, from `(type, online)` of every
/// `/sys/class/power_supply/*` entry.
///
/// Any non-battery supply (`Mains`, `USB`, `USB_PD`, `Wireless`, ...) that is
/// online means external power. `Some(false)` when such supplies exist but none
/// is online; `None` when there are none to judge by.
pub(crate) fn external_power_online(supplies: &[(String, Option<bool>)]) -> Option<bool> {
    let mut adapters = supplies
        .iter()
        .filter(|(kind, _)| !kind.trim().eq_ignore_ascii_case("battery"))
        .filter_map(|(_, online)| *online)
        .peekable();
    adapters.peek()?;
    Some(adapters.any(|online| online))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn low_trip_points_are_not_cpu_limits() {
        // A 38 °C battery zone with a 50 °C critical trip is not "serious".
        assert_eq!(
            thermal_state_from_zones(&[(38.0, Some(50.0))]),
            Some(ThermalState::Nominal)
        );
    }

    #[test]
    fn cpu_zone_types_are_recognised() {
        for kind in [
            "x86_pkg_temp",
            "coretemp",
            "k10temp",
            "acpitz",
            "TCPU",
            "cpu-thermal",
            "cpu0-thermal",
            "cpuss-0-usr",
            "soc_thermal",
            "SoC",
            "mtktscpu",
        ] {
            assert!(is_cpu_zone_type(kind), "{kind} should be a CPU zone");
        }
        for kind in [
            "battery",
            "BAT0",
            "skin-therm",
            "iwlwifi_1",
            "INT3400 Thermal",
            "pch_cannonlake",
            "gpu-thermal",
            "charger-therm",
        ] {
            assert!(!is_cpu_zone_type(kind), "{kind} should not be a CPU zone");
        }
    }

    #[test]
    fn cool_laptop_ignores_hot_non_cpu_zones() {
        let zones = vec![
            ("x86_pkg_temp".to_string(), (45.0, Some(100.0))),
            ("acpitz".to_string(), (40.0, Some(110.0))),
            // Battery and Wi-Fi zones with low trips must not count.
            ("battery".to_string(), (38.0, Some(45.0))),
            ("iwlwifi_1".to_string(), (52.0, Some(60.0))),
        ];
        let selected = select_cpu_zones(zones);
        assert_eq!(selected.len(), 2);
        assert_eq!(
            thermal_state_from_zones(&selected),
            Some(ThermalState::Nominal)
        );
    }

    #[test]
    fn unknown_zone_names_fall_back_to_all_zones() {
        let zones = vec![("vendor-a".to_string(), 1), ("vendor-b".to_string(), 2)];
        assert_eq!(select_cpu_zones(zones), vec![1, 2]);
    }

    fn supply(kind: &str, online: Option<bool>) -> (String, Option<bool>) {
        (kind.to_string(), online)
    }

    #[test]
    fn external_power_from_power_supplies() {
        // Laptop on AC, battery full (state "Full"/"Not charging").
        assert_eq!(
            external_power_online(&[supply("Battery", None), supply("Mains", Some(true))]),
            Some(true)
        );
        // Laptop on battery.
        assert_eq!(
            external_power_online(&[supply("Battery", None), supply("Mains", Some(false))]),
            Some(false)
        );
        // USB-C charger: the Mains adapter is offline but a USB PD source is online.
        assert_eq!(
            external_power_online(&[
                supply("Battery", None),
                supply("Mains", Some(false)),
                supply("USB", Some(true)),
            ]),
            Some(true)
        );
        // Only batteries (or no supplies at all): unknown.
        assert_eq!(
            external_power_online(&[supply("Battery", Some(true))]),
            None
        );
        assert_eq!(external_power_online(&[]), None);
        // Adapters without a readable `online` don't count.
        assert_eq!(external_power_online(&[supply("Mains", None)]), None);
    }
}
