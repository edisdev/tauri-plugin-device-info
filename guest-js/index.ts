import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import type { DeviceInfoResponse, BatteryInfo, NetworkInfo, StorageInfo, DisplayInfo, PowerProfile, DeviceTier, WatchOptions, UnwatchFn } from './type'

// Re-export types for consumers
export type {
  DeviceInfoResponse,
  DeviceInfoResponse as DeviceInfo,
  BatteryInfo,
  NetworkInfo,
  StorageInfo,
  DisplayInfo,
  ThermalState,
  PowerProfile,
  PowerProfileKind,
  PowerProfileReason,
  HardwareInfo,
  DeviceTier,
  DeviceTierKind,
  WatchOptions,
  UnwatchFn
} from './type'

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

// ============================================================================
// Adaptive signals
// ============================================================================

/**
 * Get a single `full | balanced | saver` recommendation that combines battery
 * level, charging state, OS power-saving mode and thermal pressure.
 *
 * @example
 * const { profile } = await getPowerProfile()
 * if (profile === 'saver') disableBackgroundSync()
 */
export async function getPowerProfile(): Promise<PowerProfile> {
  return await invoke<PowerProfile>('plugin:device-info|get_power_profile')
}

/**
 * Classify the device as `low | mid | high` from its CPU and memory, scored
 * against devices of the same form factor (desktop vs. mobile).
 *
 * @example
 * const { tier } = await getDeviceTier()
 * document.body.dataset.tier = tier // style with [data-tier="low"] { ... }
 */
export async function getDeviceTier(): Promise<DeviceTier> {
  return await invoke<DeviceTier>('plugin:device-info|get_device_tier')
}

// ============================================================================
// Reactive watch API
// ============================================================================

type WatchKind = 'battery' | 'network' | 'storage' | 'display' | 'device' | 'power'

/**
 * Subscribes to a device-info kind and invokes `callback` whenever the value changes.
 * The current value is delivered immediately, then on every change.
 * Returns a function that stops watching and removes the listener.
 */
async function watch<T>(
  kind: WatchKind,
  callback: (value: T) => void,
  options?: WatchOptions
): Promise<UnwatchFn> {
  // Listen before starting so no change emitted after the start is missed.
  let changed = false
  const unlisten = await listen<T>(`device-info://${kind}-changed`, (event) => {
    changed = true
    callback(event.payload)
  })

  let current: T | null
  try {
    // Returns the current value for this subscriber only; the backend emits
    // events only on change, so other subscribers don't get a duplicate.
    current = await invoke<T | null>('plugin:device-info|start_watching', {
      kind,
      intervalMs: options?.intervalMs
    })
  } catch (error) {
    // Don't leak the event listener if the backend failed to start the watcher.
    unlisten()
    throw error
  }

  // A change event that already arrived is at least as new as `current`;
  // delivering `current` after it would roll the caller back to a stale value.
  if (current !== null && current !== undefined && !changed) {
    callback(current)
  }

  return async () => {
    unlisten()
    await invoke('plugin:device-info|stop_watching', { kind })
  }
}

/** Watch battery status (level, charging state, health) for changes. */
export function watchBattery(callback: (info: BatteryInfo) => void, options?: WatchOptions): Promise<UnwatchFn> {
  return watch<BatteryInfo>('battery', callback, options)
}

/** Watch network connectivity (IP, type, MAC) for changes — e.g. Wi-Fi ↔ offline. */
export function watchNetwork(callback: (info: NetworkInfo) => void, options?: WatchOptions): Promise<UnwatchFn> {
  return watch<NetworkInfo>('network', callback, options)
}

/** Watch storage capacity (total/free space) for changes. */
export function watchStorage(callback: (info: StorageInfo) => void, options?: WatchOptions): Promise<UnwatchFn> {
  return watch<StorageInfo>('storage', callback, options)
}

/** Watch display properties (resolution, scale, refresh rate) for changes — e.g. plugging in a monitor. */
export function watchDisplay(callback: (info: DisplayInfo) => void, options?: WatchOptions): Promise<UnwatchFn> {
  return watch<DisplayInfo>('display', callback, options)
}

/** Watch device identity fields for changes (rarely changes; useful on mobile). */
export function watchDevice(callback: (info: DeviceInfoResponse) => void, options?: WatchOptions): Promise<UnwatchFn> {
  return watch<DeviceInfoResponse>('device', callback, options)
}

/**
 * Watch the power profile — fires when the profile, any reason, or the
 * underlying battery/thermal/low-power readings change.
 *
 * @example
 * const unwatch = await watchPowerProfile(({ profile }) => {
 *   animations.enabled = profile === 'full'
 * })
 */
export function watchPowerProfile(callback: (profile: PowerProfile) => void, options?: WatchOptions): Promise<UnwatchFn> {
  return watch<PowerProfile>('power', callback, options)
}
