# API Reference

This plugin provides 5 getters to read device information, a
[reactive watch API](/api/watch) to subscribe to changes, and
[adaptive signals](/api/adaptive) that turn readings into decisions.

## Functions Overview

| Function | Description |
|----------|-------------|
| [`getDeviceInfo()`](/api/device-info) | Device identification and hardware info |
| [`getBatteryInfo()`](/api/battery-info) | Battery status and health |
| [`getNetworkInfo()`](/api/network-info) | Network connection details |
| [`getStorageInfo()`](/api/storage-info) | Storage capacity information |
| [`getDisplayInfo()`](/api/display-info) | Display properties and capabilities |
| [`watch*()`](/api/watch) | Subscribe to changes for any of the above kinds |
| [`getPowerProfile()`](/api/adaptive#power-profile) | `full` / `balanced` / `saver` from battery, power saving and thermal state |
| [`getDeviceTier()`](/api/adaptive#device-tier) | `low` / `mid` / `high` hardware capability class |

## Quick Reference

```typescript
import { 
  getDeviceInfo,    // → DeviceInfoResponse
  getBatteryInfo,   // → BatteryInfo
  getNetworkInfo,   // → NetworkInfo
  getStorageInfo,   // → StorageInfo
  getDisplayInfo,   // → DisplayInfo
  getPowerProfile,  // → PowerProfile
  getDeviceTier,    // → DeviceTier

  // Reactive watch API → Promise<UnwatchFn>
  watchDevice,
  watchBattery,
  watchNetwork,
  watchStorage,
  watchDisplay,
  watchPowerProfile
} from 'tauri-plugin-device-info-api';
```

## Type Exports

All response types are exported for TypeScript users:

```typescript
import type { 
  DeviceInfoResponse, 
  BatteryInfo, 
  NetworkInfo, 
  StorageInfo, 
  DisplayInfo,
  PowerProfile,
  DeviceTier,
  WatchOptions,
  UnwatchFn
} from 'tauri-plugin-device-info-api';
```

## Naming Convention

The JavaScript API uses **camelCase** for all property names:

| Rust (internal) | JavaScript |
|-----------------|------------|
| `is_charging` | `isCharging` |
| `ip_address` | `ipAddress` |
| `mac_address` | `macAddress` |
| `total_space` | `totalSpace` |
| `free_space` | `freeSpace` |
| `storage_type` | `storageType` |
| `scale_factor` | `scaleFactor` |
| `refresh_rate` | `refreshRate` |


