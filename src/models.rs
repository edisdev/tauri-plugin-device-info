//! Data models for the device-info plugin.
//!
//! This module contains all the response types returned by the plugin's API.
//! All types implement `Serialize` and `Deserialize` for JSON communication
//! with the frontend, and use `camelCase` naming for JavaScript compatibility.

use serde::{Deserialize, Serialize};

// ============================================================================
// Device Information
// ============================================================================

/// Comprehensive device identification and hardware information.
///
/// Contains unique identifiers and basic hardware details that can be used
/// for device fingerprinting, analytics, or user identification.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DeviceInfoResponse {
    /// Unique device identifier (Hardware UUID or GUID)
    pub uuid: Option<String>,
    /// Device manufacturer (e.g., "Apple Inc.", "Dell Inc.")
    pub manufacturer: Option<String>,
    /// Device model name (e.g., "MacBook Pro", "OptiPlex 7090")
    pub model: Option<String>,
    /// Device serial number (may be restricted on some platforms)
    pub serial: Option<String>,
    /// Android-specific device ID (Android only)
    pub android_id: Option<String>,
    /// User-assigned device name (hostname)
    pub device_name: Option<String>,
}

// ============================================================================
// Battery Information
// ============================================================================

/// Battery status and health information.
///
/// Provides real-time battery metrics including charge level, charging state,
/// and overall battery health assessment.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryInfo {
    /// Current battery charge level (0-100 percentage)
    pub level: Option<f32>,
    /// Whether the device is currently connected to power and charging
    pub is_charging: Option<bool>,
    /// Battery health status (e.g., "Good", "Fair", "Poor")
    pub health: Option<String>,
}

// ============================================================================
// Network Information
// ============================================================================

/// Network connection details and identifiers.
///
/// Contains information about the current network connection including
/// IP address, connection type, and hardware address.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    /// Local IPv4 address (e.g., "192.168.1.100")
    pub ip_address: Option<String>,
    /// Connection type: "wifi", "ethernet", "cellular", or "unknown"
    pub network_type: Option<String>,
    /// MAC address (unavailable on iOS/Android due to privacy restrictions)
    pub mac_address: Option<String>,
}

// ============================================================================
// Storage Information
// ============================================================================

/// Storage capacity and type information.
///
/// Provides information about the primary storage device including
/// total capacity, available space, and storage technology type.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    /// Total storage capacity in bytes
    pub total_space: u64,
    /// Available (free) storage space in bytes
    pub free_space: u64,
    /// Storage technology type: "Ssd", "Hdd", "Removable", "Unknown"
    pub storage_type: Option<String>,
}

// ============================================================================
// Display Information
// ============================================================================

/// Display/screen properties and capabilities.
///
/// Contains information about the primary display including resolution,
/// scaling factor, and refresh rate.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    /// Screen width in physical pixels
    pub width: u32,
    /// Screen height in physical pixels
    pub height: u32,
    /// Display scale factor (e.g., 2.0 for Retina/HiDPI displays)
    pub scale_factor: f64,
    /// Screen refresh rate in Hz (e.g., 60.0, 120.0, 144.0)
    pub refresh_rate: Option<f64>,
}

// ============================================================================
// Power State & Power Profile
// ============================================================================

/// How hot the device is running, using Apple's four-level scale.
///
/// Platforms with finer scales are mapped onto it (e.g. Android's
/// `THERMAL_STATUS_SEVERE` → [`ThermalState::Serious`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThermalState {
    /// Normal operating temperature.
    Nominal,
    /// Slightly elevated; consider trimming background work.
    Fair,
    /// High; the OS is throttling. Reduce CPU/GPU work.
    Serious,
    /// Very high; the system may shut work down. Do the minimum.
    Critical,
}

/// Raw power-related OS signals, as reported by the platform.
///
/// Fields are `None` when the platform doesn't expose the signal.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerState {
    /// Whether the OS power-saving mode is on (Low Power Mode, Battery Saver, `low-power` profile).
    pub low_power_mode: Option<bool>,
    /// Current thermal pressure.
    pub thermal_state: Option<ThermalState>,
    /// Whether the device is running on external power (AC adapter, USB-C
    /// charger), even if the battery is not charging — e.g. full, or held at a
    /// charge limit. `None` when the platform doesn't say.
    pub on_external_power: Option<bool>,
}

/// The recommended amount of work an app should do right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerProfileKind {
    /// No constraints — run at full quality.
    Full,
    /// Some pressure — skip non-essential work, lower animation/refresh rates.
    Balanced,
    /// Heavy pressure — do the minimum; pause background work and effects.
    Saver,
}

/// Why a [`PowerProfile`] is not [`PowerProfileKind::Full`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PowerProfileReason {
    /// The OS power-saving mode is on. → `saver`
    LowPowerMode,
    /// Thermal state is `critical`. → `saver`
    ThermalCritical,
    /// Thermal state is `serious`. → `saver`
    ThermalSerious,
    /// Thermal state is `fair`. → `balanced`
    ThermalFair,
    /// On battery at or below the critical threshold (20%). → `saver`
    BatteryCritical,
    /// On battery at or below the low threshold (40%). → `balanced`
    BatteryLow,
}

/// A single, actionable power signal combining battery, power-saving mode and
/// thermal state.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerProfile {
    /// The recommended profile.
    pub profile: PowerProfileKind,
    /// Every condition that pushed the profile below `full` (empty when `full`).
    pub reasons: Vec<PowerProfileReason>,
    /// Battery level (0-100), or `None` on devices without a battery or when
    /// the platform reported an implausible/unknown level.
    pub battery_level: Option<f32>,
    /// Whether the device is charging.
    pub is_charging: Option<bool>,
    /// Whether the OS power-saving mode is on.
    pub low_power_mode: Option<bool>,
    /// Current thermal pressure.
    pub thermal_state: Option<ThermalState>,
    /// Whether the device is on external power, even when not charging.
    /// `None` when the platform doesn't say.
    pub on_external_power: Option<bool>,
}

// ============================================================================
// Hardware & Device Tier
// ============================================================================

/// Hardware capability signals used to classify the device.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    /// Logical CPU core count.
    pub cpu_cores: Option<u32>,
    /// Physical CPU core count.
    pub physical_cores: Option<u32>,
    /// CPU / SoC name (e.g. "Apple M2", "Intel(R) Core(TM) i7-1165G7", "Tensor G3").
    pub cpu_model: Option<String>,
    /// Highest CPU clock in MHz, when the platform exposes it.
    pub cpu_max_frequency_mhz: Option<u64>,
    /// Total physical memory in bytes.
    pub total_memory: Option<u64>,
    /// `true` on phones/tablets — tiers are scored against mobile hardware.
    pub is_mobile: bool,
    /// Android only: the OS flagged this as a low-RAM device.
    pub is_low_ram_device: Option<bool>,
    /// Android 12+ only: the declared media performance class (e.g. 31, 33), if any.
    pub media_performance_class: Option<u32>,
}

/// Coarse device capability class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceTierKind {
    Low,
    Mid,
    High,
}

/// How capable this device is, relative to others of its form factor.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTier {
    /// The capability class.
    pub tier: DeviceTierKind,
    /// Capability score from 0 (weakest) to 100 (strongest).
    pub score: u8,
    /// The hardware signals the score was computed from.
    pub hardware: HardwareInfo,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ============ DeviceInfoResponse Tests ============

    #[test]
    fn device_info_response_default_is_all_none() {
        let info = DeviceInfoResponse::default();
        assert!(info.uuid.is_none());
        assert!(info.manufacturer.is_none());
        assert!(info.model.is_none());
        assert!(info.serial.is_none());
        assert!(info.android_id.is_none());
        assert!(info.device_name.is_none());
    }

    #[test]
    fn device_info_response_serializes_correctly() {
        let info = DeviceInfoResponse {
            uuid: Some("test-uuid".to_string()),
            manufacturer: Some("Apple Inc.".to_string()),
            model: Some("MacBook Pro".to_string()),
            serial: Some("ABC123".to_string()),
            android_id: None,
            device_name: Some("My Mac".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-uuid"));
        assert!(json.contains("Apple Inc."));
        assert!(json.contains("MacBook Pro"));
    }

    #[test]
    fn device_info_response_deserializes_correctly() {
        let json = r#"{
            "uuid": "12345",
            "manufacturer": "Dell",
            "model": "XPS 15",
            "serial": null,
            "android_id": null,
            "device_name": "Work Laptop"
        }"#;

        let info: DeviceInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(info.uuid, Some("12345".to_string()));
        assert_eq!(info.manufacturer, Some("Dell".to_string()));
        assert!(info.serial.is_none());
    }

    // ============ BatteryInfo Tests ============

    #[test]
    fn battery_info_default_is_all_none() {
        let battery = BatteryInfo::default();
        assert!(battery.level.is_none());
        assert!(battery.is_charging.is_none());
        assert!(battery.health.is_none());
    }

    #[test]
    fn battery_info_serializes_with_camel_case() {
        let battery = BatteryInfo {
            level: Some(85.0),
            is_charging: Some(true),
            health: Some("Good".to_string()),
        };

        let json = serde_json::to_string(&battery).unwrap();
        // Check camelCase: isCharging not is_charging
        assert!(json.contains("isCharging"));
        assert!(!json.contains("is_charging"));
        assert!(json.contains("85"));
    }

    #[test]
    fn battery_info_deserializes_from_camel_case() {
        let json = r#"{"level": 50.0, "isCharging": false, "health": "Good"}"#;
        let battery: BatteryInfo = serde_json::from_str(json).unwrap();

        assert_eq!(battery.level, Some(50.0));
        assert_eq!(battery.is_charging, Some(false));
    }

    // ============ NetworkInfo Tests ============

    #[test]
    fn network_info_default_is_all_none() {
        let network = NetworkInfo::default();
        assert!(network.ip_address.is_none());
        assert!(network.network_type.is_none());
        assert!(network.mac_address.is_none());
    }

    #[test]
    fn network_info_serializes_with_camel_case() {
        let network = NetworkInfo {
            ip_address: Some("192.168.1.100".to_string()),
            network_type: Some("wifi".to_string()),
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
        };

        let json = serde_json::to_string(&network).unwrap();
        assert!(json.contains("ipAddress"));
        assert!(json.contains("networkType"));
        assert!(json.contains("macAddress"));
    }

    // ============ StorageInfo Tests ============

    #[test]
    fn storage_info_default_has_zero_values() {
        let storage = StorageInfo::default();
        assert_eq!(storage.total_space, 0);
        assert_eq!(storage.free_space, 0);
        assert!(storage.storage_type.is_none());
    }

    #[test]
    fn storage_info_serializes_with_camel_case() {
        let storage = StorageInfo {
            total_space: 500_000_000_000, // 500GB
            free_space: 100_000_000_000,  // 100GB
            storage_type: Some("Ssd".to_string()),
        };

        let json = serde_json::to_string(&storage).unwrap();
        assert!(json.contains("totalSpace"));
        assert!(json.contains("freeSpace"));
        assert!(json.contains("storageType"));
    }

    // ============ DisplayInfo Tests ============

    #[test]
    fn display_info_default_has_zero_values() {
        let display = DisplayInfo::default();
        assert_eq!(display.width, 0);
        assert_eq!(display.height, 0);
        assert_eq!(display.scale_factor, 0.0);
        assert!(display.refresh_rate.is_none());
    }

    #[test]
    fn display_info_serializes_with_camel_case() {
        let display = DisplayInfo {
            width: 2560,
            height: 1440,
            scale_factor: 2.0,
            refresh_rate: Some(60.0),
        };

        let json = serde_json::to_string(&display).unwrap();
        assert!(json.contains("scaleFactor"));
        assert!(json.contains("refreshRate"));
        assert!(json.contains("2560"));
    }

    #[test]
    fn display_info_deserializes_from_camel_case() {
        let json = r#"{"width": 1920, "height": 1080, "scaleFactor": 1.0, "refreshRate": 144.0}"#;
        let display: DisplayInfo = serde_json::from_str(json).unwrap();

        assert_eq!(display.width, 1920);
        assert_eq!(display.height, 1080);
        assert_eq!(display.refresh_rate, Some(144.0));
    }
}
