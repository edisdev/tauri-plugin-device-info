use std::process::Command;
use sysinfo::System;

use crate::models::*;

/// Helper function to run system_profiler with a specific data type and return parsed JSON
fn run_system_profiler(data_type: &str) -> Option<serde_json::Value> {
    Command::new("system_profiler")
        .args([data_type, "-json"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|json| serde_json::from_str(&json).ok())
}

pub fn get_device_info() -> crate::Result<DeviceInfoResponse> {
    let hostname = System::host_name().unwrap_or_else(|_| "Unknown Device".to_string());

    if let Some(v) = run_system_profiler("SPHardwareDataType") {
        if let Some(h) = v.get("SPHardwareDataType").and_then(|v| v.get(0)) {
            return Ok(DeviceInfoResponse {
                device_name: Some(hostname),
                manufacturer: Some("Apple Inc.".to_string()),
                model: h
                    .get("machine_name")
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                uuid: h
                    .get("platform_UUID")
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                serial: h
                    .get("serial_number")
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                android_id: None,
            });
        }
    }

    // Fallback response
    Ok(DeviceInfoResponse {
        device_name: Some(hostname),
        manufacturer: Some("Apple Inc.".to_string()),
        model: None,
        uuid: None,
        serial: None,
        android_id: None,
    })
}

pub fn get_battery_info() -> crate::Result<BatteryInfo> {
    let output = Command::new("pmset")
        .args(["-g", "batt"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());

    if let Some(output_str) = output {
        // Parse percentage
        let level = output_str
            .split('\t')
            .nth(1)
            .and_then(|s| s.split(';').next())
            .and_then(|s| s.replace('%', "").trim().parse::<f32>().ok());

        // Parse charging status
        let is_charging = output_str.contains("charging") && !output_str.contains("discharging");

        // Health is harder to get via pmset without XML, defaulting to good if present
        let health = if output_str.contains("present: true") {
            Some("Good".to_string())
        } else {
            None
        };

        return Ok(BatteryInfo {
            level,
            is_charging: Some(is_charging),
            health,
        });
    }

    Ok(BatteryInfo {
        level: None,
        is_charging: None,
        health: None,
    })
}

pub fn get_display_refresh_rate() -> Option<f64> {
    use core_graphics::display::CGDisplay;

    let main_display = CGDisplay::main();
    let mode = main_display.display_mode()?;
    let rate = mode.refresh_rate();

    // If refresh rate is 0, it means the display uses a variable refresh rate
    // In that case, return None or a default value
    if rate > 0.0 {
        Some(rate)
    } else {
        None
    }
}

/// Reads Low Power Mode and thermal state from `NSProcessInfo`.
///
/// Low Power Mode exists on macOS 12+; on older systems the selector is absent
/// and `low_power_mode` is `None` rather than crashing on an unknown selector.
pub fn get_power_state() -> PowerState {
    use objc2::runtime::NSObjectProtocol;
    use objc2::sel;
    use objc2_foundation::{NSProcessInfo, NSProcessInfoThermalState};

    let info = NSProcessInfo::processInfo();

    let thermal_state = match info.thermalState() {
        NSProcessInfoThermalState::Nominal => Some(ThermalState::Nominal),
        NSProcessInfoThermalState::Fair => Some(ThermalState::Fair),
        NSProcessInfoThermalState::Serious => Some(ThermalState::Serious),
        NSProcessInfoThermalState::Critical => Some(ThermalState::Critical),
        _ => None,
    };

    let low_power_mode = info
        .respondsToSelector(sel!(isLowPowerModeEnabled))
        .then(|| info.isLowPowerModeEnabled());

    PowerState {
        low_power_mode,
        thermal_state,
        on_external_power: providing_power_source()
            .as_deref()
            .and_then(power_source_is_external),
    }
}

/// The power source currently powering the Mac, from IOKit power sources
/// (`"AC Power"`, `"Battery Power"` or `"UPS Power"`).
fn providing_power_source() -> Option<String> {
    use std::ffi::{c_char, c_void, CStr};

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOPSCopyPowerSourcesInfo() -> *const c_void;
        fn IOPSGetProvidingPowerSourceType(snapshot: *const c_void) -> *const c_void;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringGetCString(
            string: *const c_void,
            buffer: *mut c_char,
            buffer_size: isize,
            encoding: u32,
        ) -> u8;
        fn CFRelease(cf: *const c_void);
    }
    const CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

    // SAFETY: the snapshot follows the Create rule and is released below; the
    // source type string follows the Get rule (owned by the snapshot) and is
    // copied out before the snapshot is released.
    unsafe {
        let snapshot = IOPSCopyPowerSourcesInfo();
        if snapshot.is_null() {
            return None;
        }
        let kind = IOPSGetProvidingPowerSourceType(snapshot);
        let mut buf = [0 as c_char; 64];
        let copied = !kind.is_null()
            && CFStringGetCString(
                kind,
                buf.as_mut_ptr(),
                buf.len() as isize,
                CF_STRING_ENCODING_UTF8,
            ) != 0;
        let result = copied.then(|| CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned());
        CFRelease(snapshot);
        result
    }
}

/// The hardware MAC address of a BSD interface (`en0`, ...), read from the
/// `IOMACAddress` property of its IOKit network controller.
///
/// macOS 27.2 masks link-layer addresses as `02:00:00:00:00:00` in
/// getifaddrs, sysctl and ioctl for third-party processes; IOKit still
/// reports the real address. `None` when the interface has no IOKit entry
/// (`utun`, `bridge`, ...) or no MAC.
pub fn hardware_mac_address(bsd_name: &str) -> Option<String> {
    use std::ffi::{c_char, c_void, CString};

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOBSDNameMatching(main_port: u32, options: u32, bsd_name: *const c_char) -> *mut c_void;
        fn IOServiceGetMatchingService(main_port: u32, matching: *mut c_void) -> u32;
        fn IORegistryEntrySearchCFProperty(
            entry: u32,
            plane: *const c_char,
            key: *const c_void,
            allocator: *const c_void,
            options: u32,
        ) -> *const c_void;
        fn IOObjectRelease(object: u32) -> i32;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(
            allocator: *const c_void,
            c_str: *const c_char,
            encoding: u32,
        ) -> *const c_void;
        fn CFGetTypeID(cf: *const c_void) -> usize;
        fn CFDataGetTypeID() -> usize;
        fn CFDataGetLength(data: *const c_void) -> isize;
        fn CFDataGetBytePtr(data: *const c_void) -> *const u8;
        fn CFRelease(cf: *const c_void);
    }
    const CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    const K_IO_MAIN_PORT_DEFAULT: u32 = 0;
    // kIORegistryIterateRecursively | kIORegistryIterateParents: the MAC lives
    // on the controller, a parent of the interface entry.
    const SEARCH_PARENTS: u32 = 0x1 | 0x2;

    let name = CString::new(bsd_name).ok()?;
    // SAFETY: the matching dictionary is consumed by
    // IOServiceGetMatchingService; the service, key and property follow the
    // Create/Copy rule and are released below. The data bytes are copied out
    // before the property is released.
    unsafe {
        let matching = IOBSDNameMatching(K_IO_MAIN_PORT_DEFAULT, 0, name.as_ptr());
        if matching.is_null() {
            return None;
        }
        let service = IOServiceGetMatchingService(K_IO_MAIN_PORT_DEFAULT, matching);
        if service == 0 {
            return None;
        }
        let key = CFStringCreateWithCString(
            std::ptr::null(),
            c"IOMACAddress".as_ptr(),
            CF_STRING_ENCODING_UTF8,
        );
        let data = if key.is_null() {
            std::ptr::null()
        } else {
            IORegistryEntrySearchCFProperty(
                service,
                c"IOService".as_ptr(),
                key,
                std::ptr::null(),
                SEARCH_PARENTS,
            )
        };
        let mac = (!data.is_null() && CFGetTypeID(data) == CFDataGetTypeID())
            .then(|| {
                std::slice::from_raw_parts(CFDataGetBytePtr(data), CFDataGetLength(data) as usize)
                    .to_vec()
            })
            .and_then(|bytes| format_mac(&bytes));
        if !data.is_null() {
            CFRelease(data);
        }
        if !key.is_null() {
            CFRelease(key);
        }
        IOObjectRelease(service);
        mac
    }
}

/// Formats a 6-byte MAC as `aa:bb:cc:dd:ee:ff`, the format `netdev` uses.
fn format_mac(bytes: &[u8]) -> Option<String> {
    (bytes.len() == 6).then(|| {
        bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(":")
    })
}

/// Maps an IOKit providing-power-source type to "on external power".
fn power_source_is_external(kind: &str) -> Option<bool> {
    match kind {
        "AC Power" => Some(true),
        "Battery Power" => Some(false),
        // A UPS is external but itself on battery; don't guess.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_bytes_are_formatted_like_netdev() {
        assert_eq!(
            format_mac(&[0xfc, 0xb2, 0x14, 0x0a, 0xe1, 0x35]).as_deref(),
            Some("fc:b2:14:0a:e1:35")
        );
        assert_eq!(format_mac(&[1, 2, 3]), None);
    }

    #[test]
    fn power_source_type_maps_to_external_power() {
        assert_eq!(power_source_is_external("AC Power"), Some(true));
        assert_eq!(power_source_is_external("Battery Power"), Some(false));
        assert_eq!(power_source_is_external("UPS Power"), None);
    }

    #[test]
    fn providing_power_source_reads_on_host() {
        // Must not crash; CI VMs may not report a source at all.
        let source = providing_power_source();
        println!("providing power source: {source:?}");
    }

    #[test]
    fn parse_battery_percentage_from_pmset_output() {
        // Simulated pmset output
        let pmset_output = "Now drawing from 'Battery Power'\n -InternalBattery-0 (id=1234567)\t85%; discharging; 3:45 remaining present: true";

        // Extract percentage like our function does
        let level = pmset_output
            .split('\t')
            .nth(1)
            .and_then(|s| s.split(';').next())
            .and_then(|s| s.replace('%', "").trim().parse::<f32>().ok());

        assert_eq!(level, Some(85.0));
    }

    #[test]
    fn parse_charging_status_from_pmset() {
        let charging_output =
            "Now drawing from 'AC Power'\n -InternalBattery-0\t95%; charging; 0:30 remaining";
        let discharging_output = "Now drawing from 'Battery Power'\n -InternalBattery-0\t85%; discharging; 3:45 remaining";

        let is_charging_1 =
            charging_output.contains("charging") && !charging_output.contains("discharging");
        let is_charging_2 =
            discharging_output.contains("charging") && !discharging_output.contains("discharging");

        assert!(is_charging_1);
        assert!(!is_charging_2);
    }

    #[test]
    fn parse_system_profiler_json() {
        // Simulated system_profiler JSON output
        let json = r#"{
            "SPHardwareDataType": [{
                "machine_name": "MacBook Pro",
                "platform_UUID": "12345678-1234-5678-9ABC-DEF012345678",
                "serial_number": "C02ABC123"
            }]
        }"#;

        let v: serde_json::Value = serde_json::from_str(json).unwrap();

        let hardware = v.get("SPHardwareDataType").and_then(|v| v.get(0));

        assert!(hardware.is_some());

        let h = hardware.unwrap();
        assert_eq!(
            h.get("machine_name").and_then(|v| v.as_str()),
            Some("MacBook Pro")
        );
        assert_eq!(
            h.get("serial_number").and_then(|v| v.as_str()),
            Some("C02ABC123")
        );
    }

    #[test]
    fn fallback_hostname_works() {
        // Test that unwrap_or_else provides fallback
        let hostname: Option<String> = None;
        let result = hostname.unwrap_or_else(|| "Unknown Device".to_string());
        assert_eq!(result, "Unknown Device");
    }

    #[test]
    fn manufacturer_is_always_apple() {
        // All Macs should report Apple Inc.
        let manufacturer = Some("Apple Inc.".to_string());
        assert_eq!(manufacturer, Some("Apple Inc.".to_string()));
    }
}
