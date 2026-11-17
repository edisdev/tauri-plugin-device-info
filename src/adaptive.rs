//! Adaptive signals: turn raw device readings into decisions.
//!
//! - [`power_profile`] folds battery, power-saving mode and thermal state into a
//!   single `full | balanced | saver` recommendation.
//! - [`device_tier`] scores hardware into a `low | mid | high` capability class.
//!
//! Both are pure functions so the policy is identical on every platform and is
//! fully unit-tested; platforms only supply the raw readings.

use crate::models::*;

// ============================================================================
// Power profile
// ============================================================================

/// On battery at or below this level → `saver`.
pub const BATTERY_CRITICAL_LEVEL: f32 = 20.0;
/// On battery at or below this level → `balanced`.
pub const BATTERY_LOW_LEVEL: f32 = 40.0;

impl PowerProfileReason {
    /// The profile this condition implies on its own.
    fn implied_profile(self) -> PowerProfileKind {
        match self {
            Self::LowPowerMode
            | Self::ThermalCritical
            | Self::ThermalSerious
            | Self::BatteryCritical => PowerProfileKind::Saver,
            Self::ThermalFair | Self::BatteryLow => PowerProfileKind::Balanced,
        }
    }
}

/// Returns the battery level if it is a plausible percentage, else `None`.
///
/// Platforms report "unknown" in band: iOS `batteryLevel` is `-1` (→ `-100`
/// after scaling, always on the Simulator) and Android's
/// `BATTERY_PROPERTY_CAPACITY` is `Integer.MIN_VALUE` when unsupported. Such
/// readings must not be mistaken for an empty battery.
pub fn plausible_battery_level(level: Option<f32>) -> Option<f32> {
    level.filter(|l| (0.0..=100.0).contains(l))
}

/// Whether battery thresholds should apply: the device is not charging and is
/// not known to be on external power.
///
/// `is_charging` alone is not enough — a laptop on AC that is full, held at a
/// charge limit, or "not charging" reports `false` yet is not draining.
/// Unknown signals are treated as on battery (the conservative default).
fn on_battery(battery: &BatteryInfo, power: &PowerState) -> bool {
    battery.is_charging != Some(true) && power.on_external_power != Some(true)
}

/// Combines battery and power-state readings into a [`PowerProfile`].
///
/// The profile is the most restrictive one implied by any active condition.
/// Battery thresholds only apply while running on battery (not charging and
/// not on external power); a device with no battery or an unknown level
/// (`level == None`, or outside 0-100) is treated as on mains power.
pub fn power_profile(battery: &BatteryInfo, power: &PowerState) -> PowerProfile {
    let mut reasons = Vec::new();

    if power.low_power_mode == Some(true) {
        reasons.push(PowerProfileReason::LowPowerMode);
    }

    match power.thermal_state {
        Some(ThermalState::Critical) => reasons.push(PowerProfileReason::ThermalCritical),
        Some(ThermalState::Serious) => reasons.push(PowerProfileReason::ThermalSerious),
        Some(ThermalState::Fair) => reasons.push(PowerProfileReason::ThermalFair),
        Some(ThermalState::Nominal) | None => {}
    }

    let battery_level = plausible_battery_level(battery.level);
    if let (Some(level), true) = (battery_level, on_battery(battery, power)) {
        if level <= BATTERY_CRITICAL_LEVEL {
            reasons.push(PowerProfileReason::BatteryCritical);
        } else if level <= BATTERY_LOW_LEVEL {
            reasons.push(PowerProfileReason::BatteryLow);
        }
    }

    let profile = reasons
        .iter()
        .map(|r| r.implied_profile())
        .max()
        .unwrap_or(PowerProfileKind::Full);

    PowerProfile {
        profile,
        reasons,
        battery_level,
        is_charging: battery.is_charging,
        low_power_mode: power.low_power_mode,
        thermal_state: power.thermal_state,
        on_external_power: power.on_external_power,
    }
}

// ============================================================================
// Device tier
// ============================================================================

/// Scores below this are `low`.
pub const TIER_MID_MIN_SCORE: u8 = 30;
/// Scores at or above this are `high`.
pub const TIER_HIGH_MIN_SCORE: u8 = 65;

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

/// The value range and weight for one scoring signal. Values are placed on a
/// log scale between `lo` (scores 0) and `hi` (scores 1), because doubling RAM
/// or cores matters about equally at every step.
struct Axis {
    lo: f64,
    hi: f64,
    weight: f64,
}

struct Scale {
    memory_gib: Axis,
    cores: Axis,
    frequency_mhz: Axis,
}

const DESKTOP: Scale = Scale {
    memory_gib: Axis {
        lo: 4.0,
        hi: 32.0,
        weight: 0.5,
    },
    cores: Axis {
        lo: 2.0,
        hi: 16.0,
        weight: 0.3,
    },
    frequency_mhz: Axis {
        lo: 1500.0,
        hi: 4500.0,
        weight: 0.2,
    },
};

/// Phones almost all have 6-8 cores, so cores barely separate them; memory and
/// peak clock carry the signal.
const MOBILE: Scale = Scale {
    memory_gib: Axis {
        lo: 2.0,
        hi: 12.0,
        weight: 0.55,
    },
    cores: Axis {
        lo: 4.0,
        hi: 8.0,
        weight: 0.15,
    },
    frequency_mhz: Axis {
        lo: 1400.0,
        hi: 3200.0,
        weight: 0.3,
    },
};

impl Axis {
    fn normalize(&self, value: f64) -> f64 {
        ((value.ln() - self.lo.ln()) / (self.hi.ln() - self.lo.ln())).clamp(0.0, 1.0)
    }
}

/// Classifies hardware into a [`DeviceTier`].
///
/// The score is a weighted average of memory, core count and peak CPU clock,
/// each normalized against the device's form factor. Missing signals are left
/// out and the remaining weights rescaled. On Android, OS hints override the
/// score: a low-RAM device is always `low`, and a media performance class of
/// 31+ / 33+ guarantees at least `mid` / `high`.
pub fn device_tier(hardware: HardwareInfo) -> DeviceTier {
    let scale = if hardware.is_mobile {
        &MOBILE
    } else {
        &DESKTOP
    };

    let signals = [
        (
            &scale.memory_gib,
            hardware.total_memory.map(|b| b as f64 / GIB),
        ),
        (&scale.cores, hardware.cpu_cores.map(f64::from)),
        (
            &scale.frequency_mhz,
            hardware.cpu_max_frequency_mhz.map(|f| f as f64),
        ),
    ];

    let (weighted, total_weight) = signals
        .iter()
        .filter_map(|(axis, value)| value.filter(|v| *v > 0.0).map(|v| (axis, v)))
        .fold((0.0, 0.0), |(sum, w), (axis, v)| {
            (sum + axis.normalize(v) * axis.weight, w + axis.weight)
        });

    let score = if total_weight > 0.0 {
        (weighted / total_weight * 100.0).round() as u8
    } else {
        // Nothing to go on: assume the middle rather than guessing an extreme.
        50
    };

    let mut tier = tier_for_score(score);

    if let Some(class) = hardware.media_performance_class {
        let floor = match class {
            33.. => DeviceTierKind::High,
            31.. => DeviceTierKind::Mid,
            _ => DeviceTierKind::Low,
        };
        tier = tier.max(floor);
    }
    if hardware.is_low_ram_device == Some(true) {
        tier = DeviceTierKind::Low;
    }

    DeviceTier {
        tier,
        score,
        hardware,
    }
}

fn tier_for_score(score: u8) -> DeviceTierKind {
    if score >= TIER_HIGH_MIN_SCORE {
        DeviceTierKind::High
    } else if score >= TIER_MID_MIN_SCORE {
        DeviceTierKind::Mid
    } else {
        DeviceTierKind::Low
    }
}

// ============================================================================
// Composite getters
// ============================================================================

// Defined once over the platform getters, so desktop and mobile share them;
// only `get_battery_info`, `get_power_state` and `get_hardware_info` differ.
impl<R: tauri::Runtime> crate::DeviceInfo<R> {
    /// Combines battery, power-saving mode and thermal state into a single
    /// `full | balanced | saver` recommendation. See [`power_profile`].
    pub fn get_power_profile(&self) -> crate::Result<PowerProfile> {
        Ok(power_profile(
            &self.get_battery_info()?,
            &self.get_power_state()?,
        ))
    }

    /// Classifies the device as `low | mid | high`. See [`device_tier`].
    pub fn get_device_tier(&self) -> crate::Result<DeviceTier> {
        Ok(device_tier(self.get_hardware_info()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn battery(level: Option<f32>, charging: Option<bool>) -> BatteryInfo {
        BatteryInfo {
            level,
            is_charging: charging,
            health: None,
        }
    }

    fn power(low_power: Option<bool>, thermal: Option<ThermalState>) -> PowerState {
        PowerState {
            low_power_mode: low_power,
            thermal_state: thermal,
            on_external_power: None,
        }
    }

    fn external_power(on: Option<bool>) -> PowerState {
        PowerState {
            on_external_power: on,
            ..PowerState::default()
        }
    }

    // ---------------- power_profile ----------------

    #[test]
    fn full_when_nothing_is_wrong() {
        let p = power_profile(
            &battery(Some(90.0), Some(false)),
            &power(Some(false), Some(ThermalState::Nominal)),
        );
        assert_eq!(p.profile, PowerProfileKind::Full);
        assert!(p.reasons.is_empty());
    }

    #[test]
    fn full_on_desktop_without_battery_or_signals() {
        let p = power_profile(&BatteryInfo::default(), &PowerState::default());
        assert_eq!(p.profile, PowerProfileKind::Full);
    }

    #[test]
    fn low_power_mode_forces_saver() {
        let p = power_profile(&battery(Some(100.0), Some(true)), &power(Some(true), None));
        assert_eq!(p.profile, PowerProfileKind::Saver);
        assert_eq!(p.reasons, vec![PowerProfileReason::LowPowerMode]);
    }

    #[test]
    fn thermal_states_map_to_profiles() {
        let cases = [
            (ThermalState::Nominal, PowerProfileKind::Full),
            (ThermalState::Fair, PowerProfileKind::Balanced),
            (ThermalState::Serious, PowerProfileKind::Saver),
            (ThermalState::Critical, PowerProfileKind::Saver),
        ];
        for (thermal, expected) in cases {
            let p = power_profile(&BatteryInfo::default(), &power(None, Some(thermal)));
            assert_eq!(p.profile, expected, "thermal {thermal:?}");
        }
    }

    #[test]
    fn battery_thresholds_apply_only_when_discharging() {
        let at = |level, charging| {
            power_profile(&battery(Some(level), charging), &PowerState::default())
        };

        assert_eq!(at(20.0, Some(false)).profile, PowerProfileKind::Saver);
        assert_eq!(at(21.0, Some(false)).profile, PowerProfileKind::Balanced);
        assert_eq!(at(40.0, Some(false)).profile, PowerProfileKind::Balanced);
        assert_eq!(at(41.0, Some(false)).profile, PowerProfileKind::Full);
        // Unknown charging state is treated as discharging (the safe default).
        assert_eq!(at(10.0, None).profile, PowerProfileKind::Saver);
        // Charging never triggers battery reasons.
        assert_eq!(at(5.0, Some(true)).profile, PowerProfileKind::Full);
    }

    #[test]
    fn unknown_battery_levels_are_ignored() {
        // iOS reports -1 * 100 when the level is unknown (always on the
        // Simulator); Android reports Integer.MIN_VALUE when unsupported.
        for bogus in [-100.0, -1.0, i32::MIN as f32, 100.5, f32::NAN] {
            let p = power_profile(&battery(Some(bogus), Some(false)), &PowerState::default());
            assert_eq!(p.profile, PowerProfileKind::Full, "level {bogus}");
            assert!(p.reasons.is_empty(), "level {bogus}");
            assert_eq!(p.battery_level, None, "level {bogus}");
        }
        // Bounds are inclusive.
        assert_eq!(plausible_battery_level(Some(0.0)), Some(0.0));
        assert_eq!(plausible_battery_level(Some(100.0)), Some(100.0));
        assert_eq!(plausible_battery_level(None), None);
    }

    #[test]
    fn external_power_suppresses_battery_reasons() {
        // A laptop on AC that is full or held at a charge limit reports
        // "not charging" but is not draining.
        let at = |on| power_profile(&battery(Some(15.0), Some(false)), &external_power(on));

        assert_eq!(at(Some(true)).profile, PowerProfileKind::Full);
        assert!(at(Some(true)).reasons.is_empty());
        assert_eq!(at(Some(true)).on_external_power, Some(true));
        // Known to be on battery, or unknown: thresholds apply.
        assert_eq!(at(Some(false)).profile, PowerProfileKind::Saver);
        assert_eq!(at(None).profile, PowerProfileKind::Saver);
    }

    #[test]
    fn external_power_does_not_hide_other_reasons() {
        let p = power_profile(
            &battery(Some(10.0), Some(false)),
            &PowerState {
                low_power_mode: Some(true),
                thermal_state: Some(ThermalState::Fair),
                on_external_power: Some(true),
            },
        );
        assert_eq!(p.profile, PowerProfileKind::Saver);
        assert_eq!(
            p.reasons,
            vec![
                PowerProfileReason::LowPowerMode,
                PowerProfileReason::ThermalFair
            ]
        );
    }

    #[test]
    fn most_restrictive_reason_wins_and_all_are_reported() {
        let p = power_profile(
            &battery(Some(35.0), Some(false)),
            &power(Some(false), Some(ThermalState::Serious)),
        );
        assert_eq!(p.profile, PowerProfileKind::Saver);
        assert_eq!(
            p.reasons,
            vec![
                PowerProfileReason::ThermalSerious,
                PowerProfileReason::BatteryLow
            ]
        );
    }

    #[test]
    fn power_profile_serializes_for_js() {
        let p = power_profile(
            &battery(Some(15.0), Some(false)),
            &power(Some(true), Some(ThermalState::Fair)),
        );
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["profile"], "saver");
        assert_eq!(json["thermalState"], "fair");
        assert_eq!(json["lowPowerMode"], true);
        assert_eq!(json["batteryLevel"], 15.0);
        assert!(json["onExternalPower"].is_null());
        assert_eq!(
            json["reasons"],
            serde_json::json!(["lowPowerMode", "thermalFair", "batteryCritical"])
        );
    }

    // ---------------- device_tier ----------------

    fn desktop(mem_gib: u64, cores: u32, mhz: Option<u64>) -> HardwareInfo {
        HardwareInfo {
            cpu_cores: Some(cores),
            total_memory: Some(mem_gib * 1024 * 1024 * 1024),
            cpu_max_frequency_mhz: mhz,
            ..Default::default()
        }
    }

    fn mobile(mem_gib: u64, cores: u32, mhz: Option<u64>) -> HardwareInfo {
        HardwareInfo {
            is_mobile: true,
            ..desktop(mem_gib, cores, mhz)
        }
    }

    #[test]
    fn desktop_reference_machines() {
        // Old 4 GB dual-core laptop.
        assert_eq!(
            device_tier(desktop(4, 2, Some(1800))).tier,
            DeviceTierKind::Low
        );
        // Base M1 MacBook Air (8 GB, 8 cores, no clock reported).
        assert_eq!(device_tier(desktop(8, 8, None)).tier, DeviceTierKind::Mid);
        // M3 Pro (18 GB, 12 cores).
        assert_eq!(
            device_tier(desktop(18, 12, None)).tier,
            DeviceTierKind::High
        );
        // Gaming desktop.
        assert_eq!(device_tier(desktop(32, 16, Some(5000))).score, 100);
    }

    #[test]
    fn mobile_reference_devices() {
        // Budget 2 GB phone.
        assert_eq!(
            device_tier(mobile(2, 8, Some(1600))).tier,
            DeviceTierKind::Low
        );
        // iPhone 12 (4 GB, 6 cores, no clock on iOS).
        assert_eq!(device_tier(mobile(4, 6, None)).tier, DeviceTierKind::Mid);
        // iPhone 15 Pro (8 GB, 6 cores).
        assert_eq!(device_tier(mobile(8, 6, None)).tier, DeviceTierKind::High);
        // Flagship Android (12 GB, 8 cores, 3.3 GHz).
        assert_eq!(
            device_tier(mobile(12, 8, Some(3300))).tier,
            DeviceTierKind::High
        );
    }

    #[test]
    fn same_hardware_scores_higher_on_mobile_scale() {
        let d = device_tier(desktop(6, 8, None)).score;
        let m = device_tier(mobile(6, 8, None)).score;
        assert!(m > d, "mobile {m} should outscore desktop {d}");
    }

    #[test]
    fn missing_signals_default_to_mid() {
        let t = device_tier(HardwareInfo::default());
        assert_eq!(t.score, 50);
        assert_eq!(t.tier, DeviceTierKind::Mid);
    }

    #[test]
    fn zero_values_are_ignored_not_scored_as_weakest() {
        // Some platforms report 0 MHz when the clock is unknown.
        let with_zero = device_tier(desktop(16, 8, Some(0))).score;
        let without = device_tier(desktop(16, 8, None)).score;
        assert_eq!(with_zero, without);
    }

    #[test]
    fn android_hints_override_score() {
        let low_ram = HardwareInfo {
            is_low_ram_device: Some(true),
            ..mobile(12, 8, Some(3300))
        };
        assert_eq!(device_tier(low_ram).tier, DeviceTierKind::Low);

        let mpc33 = HardwareInfo {
            media_performance_class: Some(33),
            ..mobile(2, 8, Some(1600))
        };
        assert_eq!(device_tier(mpc33).tier, DeviceTierKind::High);

        let mpc31 = HardwareInfo {
            media_performance_class: Some(31),
            ..mobile(2, 8, Some(1600))
        };
        assert_eq!(device_tier(mpc31).tier, DeviceTierKind::Mid);
    }

    #[test]
    fn device_tier_serializes_for_js() {
        let json = serde_json::to_value(device_tier(desktop(16, 8, None))).unwrap();
        assert_eq!(json["tier"], "high");
        assert!(json["score"].is_u64());
        assert_eq!(json["hardware"]["cpuCores"], 8);
        assert_eq!(json["hardware"]["isMobile"], false);
    }
}
