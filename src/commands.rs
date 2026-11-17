use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::DeviceInfoExt;
use crate::Result;

#[command]
pub(crate) async fn get_device_info<R: Runtime>(app: AppHandle<R>) -> Result<DeviceInfoResponse> {
    app.device_info().get_device_info()
}

#[command]
pub(crate) async fn get_battery_info<R: Runtime>(app: AppHandle<R>) -> Result<BatteryInfo> {
    app.device_info().get_battery_info()
}

#[command]
pub(crate) async fn get_network_info<R: Runtime>(app: AppHandle<R>) -> Result<NetworkInfo> {
    app.device_info().get_network_info()
}

#[command]
pub(crate) async fn get_storage_info<R: Runtime>(app: AppHandle<R>) -> Result<StorageInfo> {
    app.device_info().get_storage_info()
}

#[command]
pub(crate) async fn get_display_info<R: Runtime>(app: AppHandle<R>) -> Result<DisplayInfo> {
    app.device_info().get_display_info()
}

#[command]
pub(crate) async fn get_power_profile<R: Runtime>(app: AppHandle<R>) -> Result<PowerProfile> {
    app.device_info().get_power_profile()
}

#[command]
pub(crate) async fn get_device_tier<R: Runtime>(app: AppHandle<R>) -> Result<DeviceTier> {
    app.device_info().get_device_tier()
}

/// Starts watching `kind` and returns its current value (or `null` if it
/// couldn't be read), for the calling subscriber only.
#[command]
pub(crate) async fn start_watching<R: Runtime>(
    app: AppHandle<R>,
    kind: String,
    interval_ms: Option<u64>,
) -> Result<Option<serde_json::Value>> {
    crate::watcher::start(&app, &kind, interval_ms)
}

#[command]
pub(crate) async fn stop_watching<R: Runtime>(app: AppHandle<R>, kind: String) -> Result<()> {
    crate::watcher::stop(&app, &kind)
}
