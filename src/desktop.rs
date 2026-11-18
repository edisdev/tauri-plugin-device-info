//! Desktop platform implementation for the device-info plugin.
//!
//! This module provides device information retrieval for desktop platforms:
//! - **Windows**: Uses WMI (Windows Management Instrumentation)
//! - **macOS**: Uses `system_profiler` and CoreGraphics
//! - **Linux**: Reads from `/sys/class/dmi/id/` and `/proc`
//!
//! Each platform has its own submodule with platform-specific implementations.

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::models::*;

// ============================================================================
// Platform-specific modules
// ============================================================================

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

// Pure sysfs rules used by `linux`; also compiled for tests on every host.
#[cfg(any(test, target_os = "linux"))]
mod sysfs;

// ============================================================================
// Plugin initialization
// ============================================================================

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<DeviceInfo<R>> {
    Ok(DeviceInfo(app.clone()))
}

/// CPU clocks below this are sensor/unit errors, not real readings.
const MIN_PLAUSIBLE_CPU_MHZ: u64 = 200;

/// Access to the device-info APIs.
pub struct DeviceInfo<R: Runtime>(AppHandle<R>);

impl<R: Runtime> DeviceInfo<R> {
    /// Retrieves comprehensive device information including UUID, manufacturer, model, and serial number.
    ///
    /// # Platform-specific behavior
    /// - **macOS**: Uses `system_profiler` for hardware details
    /// - **Windows**: Uses WMI queries
    /// - **Linux**: Reads from `/sys/class/dmi/id/`
    pub fn get_device_info(&self) -> crate::Result<DeviceInfoResponse> {
        #[cfg(target_os = "windows")]
        return windows::get_device_info();

        #[cfg(target_os = "macos")]
        return macos::get_device_info();

        #[cfg(target_os = "linux")]
        return linux::get_device_info();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return Err(crate::Error::DeviceInfo("Unsupported platform".to_string()));
    }

    /// Retrieves battery status including charge level, charging state, and health.
    ///
    /// # Returns
    /// - `level`: Battery percentage (0-100)
    /// - `is_charging`: Whether the device is currently charging
    /// - `health`: Battery health status
    pub fn get_battery_info(&self) -> crate::Result<BatteryInfo> {
        #[cfg(target_os = "macos")]
        return macos::get_battery_info();

        #[cfg(not(target_os = "macos"))]
        {
            // `starship_battery::Manager` is `!Send` (it holds an `Rc`), so cache one per
            // thread: the watch pollers call this every few seconds.
            thread_local! {
                static MANAGER: std::cell::RefCell<Option<starship_battery::Manager>> =
                    const { std::cell::RefCell::new(None) };
            }

            MANAGER.with(|cell| {
                let mut cached = cell.borrow_mut();
                if cached.is_none() {
                    // A failure is not cached, so the next call retries.
                    *cached = Some(
                        starship_battery::Manager::new()
                            .map_err(|e| crate::Error::DeviceInfo(e.to_string()))?,
                    );
                }
                let Some(manager) = cached.as_ref() else {
                    return Ok(BatteryInfo::default());
                };
                if let Some(Ok(battery)) = manager
                    .batteries()
                    .map_err(|e| crate::Error::DeviceInfo(e.to_string()))?
                    .next()
                {
                    let level = (battery.state_of_charge().value * 100.0).round();
                    let is_charging = battery.state() == starship_battery::State::Charging;
                    let health = format!("{:?}", battery.state_of_health().value * 100.0);

                    Ok(BatteryInfo {
                        level: Some(level),
                        is_charging: Some(is_charging),
                        health: Some(health),
                    })
                } else {
                    Ok(BatteryInfo::default())
                }
            })
        }
    }

    /// Gets the MAC address of the default network interface (the one used for internet access).
    /// Uses the system's routing table to find the correct interface.
    pub fn get_default_mac_address() -> String {
        use default_net::get_default_interface;
        match get_default_interface() {
            Ok(interface) => match interface.mac_addr {
                Some(mac) => mac.to_string(),
                None => "00:00:00:00:00:00".to_string(),
            },
            Err(_) => "00:00:00:00:00:00".to_string(),
        }
    }

    /// Retrieves network information including IP address, network type, and MAC address.
    ///
    /// # Returns
    /// - `ip_address`: Local IPv4 address
    /// - `network_type`: "wifi", "ethernet", or interface name
    /// - `mac_address`: MAC address of the default interface
    pub fn get_network_info(&self) -> crate::Result<NetworkInfo> {
        use local_ip_address::local_ip;
        let ip_addr = local_ip().ok();
        let ip_str = ip_addr.map(|ip| ip.to_string());

        // Call the static helper function and convert Result to Option
        let mac_address = Some(Self::get_default_mac_address());

        let mut network_type = Some("unknown".to_string());

        if let Some(target_ip) = ip_addr {
            let networks = sysinfo::Networks::new_with_refreshed_list();

            if let Some((name, _)) = networks.iter().find(|(_, network)| {
                network
                    .ip_networks()
                    .iter()
                    .any(|net| net.addr == target_ip)
            }) {
                let name_lower = name.to_lowercase();
                if name_lower.contains("wifi") || name_lower.contains("wl") {
                    network_type = Some("wifi".to_string());
                } else if name_lower.contains("eth") || name_lower.contains("ethernet") {
                    network_type = Some("ethernet".to_string());
                } else if cfg!(target_os = "macos") && name_lower.starts_with("en") {
                    if name == "en0" {
                        network_type = Some("wifi".to_string());
                    } else {
                        network_type = Some("ethernet".to_string());
                    }
                } else {
                    network_type = Some(name.to_string());
                }
            }
        }

        Ok(NetworkInfo {
            ip_address: ip_str,
            network_type,
            mac_address,
        })
    }

    /// Retrieves storage information for the system disk.
    ///
    /// # Returns
    /// - `total_space`: Total disk capacity in bytes
    /// - `free_space`: Available disk space in bytes
    /// - `storage_type`: Disk type (SSD, HDD, etc.)
    pub fn get_storage_info(&self) -> crate::Result<StorageInfo> {
        let disks = sysinfo::Disks::new_with_refreshed_list();

        // Find the main system disk
        // Unix/Mac: root is "/"
        // Windows: typically "C:\"
        let system_disk = disks.iter().find(|d| {
            let mount = d.mount_point();
            mount == std::path::Path::new("/") || mount == std::path::Path::new("C:\\")
        });

        // Fallback: If root not found, use the disk with the largest total space
        let targeted_disk = system_disk.or_else(|| disks.iter().max_by_key(|d| d.total_space()));

        if let Some(disk) = targeted_disk {
            Ok(StorageInfo {
                total_space: disk.total_space(),
                free_space: disk.available_space(),
                // Try to get dynamic storage type (SSD, HDD, etc)
                storage_type: Some(format!("{:?}", disk.kind())),
            })
        } else {
            Ok(StorageInfo::default())
        }
    }

    pub fn get_display_info(&self) -> crate::Result<DisplayInfo> {
        let windows = self.0.webview_windows();
        // Use find to get the first available window
        if let Some(window) = windows.values().next() {
            if let Ok(Some(monitor)) = window.primary_monitor() {
                let size = monitor.size();
                let refresh_rate = {
                    #[cfg(target_os = "macos")]
                    {
                        macos::get_display_refresh_rate()
                    }
                    #[cfg(target_os = "windows")]
                    {
                        windows::get_display_refresh_rate()
                    }
                    #[cfg(target_os = "linux")]
                    {
                        linux::get_display_refresh_rate()
                    }
                    #[cfg(not(any(
                        target_os = "macos",
                        target_os = "windows",
                        target_os = "linux"
                    )))]
                    {
                        None
                    }
                };

                return Ok(DisplayInfo {
                    width: size.width,
                    height: size.height,
                    scale_factor: monitor.scale_factor(),
                    refresh_rate,
                });
            }
        }
        Ok(DisplayInfo::default())
    }
    /// Reads the OS power-saving mode and thermal state.
    ///
    /// # Platform-specific behavior
    /// - **macOS**: `NSProcessInfo` (`isLowPowerModeEnabled` needs macOS 12+)
    /// - **Windows**: Battery/Energy Saver via `GetSystemPowerStatus`; no thermal state
    /// - **Linux**: ACPI platform profile and `/sys/class/thermal` zones
    pub fn get_power_state(&self) -> crate::Result<PowerState> {
        #[cfg(target_os = "macos")]
        return Ok(macos::get_power_state());

        #[cfg(target_os = "windows")]
        return Ok(windows::get_power_state());

        #[cfg(target_os = "linux")]
        return Ok(linux::get_power_state());

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return Ok(PowerState::default());
    }

    /// Reads CPU and memory capability signals.
    pub fn get_hardware_info(&self) -> crate::Result<HardwareInfo> {
        use sysinfo::{CpuRefreshKind, RefreshKind, System};

        let sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram())
                .with_cpu(CpuRefreshKind::nothing().with_frequency()),
        );

        let cpus = sys.cpus();
        // sysinfo reports nonsense on some CPUs (e.g. `4` on Apple Silicon), so
        // anything below a plausible clock is treated as unknown.
        let sysinfo_mhz = cpus
            .iter()
            .map(|c| c.frequency())
            .max()
            .filter(|f| *f >= MIN_PLAUSIBLE_CPU_MHZ);

        #[cfg(target_os = "linux")]
        let cpu_max_frequency_mhz = linux::get_cpu_max_frequency_mhz().or(sysinfo_mhz);
        #[cfg(not(target_os = "linux"))]
        let cpu_max_frequency_mhz = sysinfo_mhz;

        Ok(HardwareInfo {
            cpu_cores: (!cpus.is_empty()).then_some(cpus.len() as u32),
            physical_cores: System::physical_core_count().map(|c| c as u32),
            cpu_model: cpus
                .first()
                .map(|c| c.brand().trim().to_string())
                .filter(|b| !b.is_empty()),
            cpu_max_frequency_mhz,
            total_memory: Some(sys.total_memory()).filter(|m| *m > 0),
            is_mobile: false,
            is_low_ram_device: None,
            media_performance_class: None,
        })
    }
}
