import { invoke } from '@tauri-apps/api/core'

import type { DeviceInfoResponse, BatteryInfo, NetworkInfo, StorageInfo, DisplayInfo } from './type'

// Re-export types for consumers
export type { DeviceInfoResponse, DeviceInfoResponse as DeviceInfo, BatteryInfo, NetworkInfo, StorageInfo, DisplayInfo } from './type'

/**
 * Get comprehensive device information including UUID, manufacturer, model, etc.
 */
export async function getDeviceInfo(): Promise<DeviceInfoResponse> {
  return await invoke<DeviceInfoResponse>('plugin:device-info|get_device_info')
}

/**
 * Get battery status including level, charging state, and health.
 */
export async function getBatteryInfo(): Promise<BatteryInfo> {
  return await invoke<BatteryInfo>('plugin:device-info|get_battery_info')
}

/**
 * Get network information including IP address, network type, and MAC address.
 */
export async function getNetworkInfo(): Promise<NetworkInfo> {
  return await invoke<NetworkInfo>('plugin:device-info|get_network_info')
}

/**
 * Get storage information including total space, free space, and storage type.
 */
export async function getStorageInfo(): Promise<StorageInfo> {
  return await invoke<StorageInfo>('plugin:device-info|get_storage_info')
}

/**
 * Get display information including resolution, scale factor, and refresh rate.
 */
export async function getDisplayInfo(): Promise<DisplayInfo> {
  return await invoke<DisplayInfo>('plugin:device-info|get_display_info')
}
