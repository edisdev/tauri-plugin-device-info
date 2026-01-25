# API Reference

This plugin provides 5 main functions to access device information.

## Functions Overview

| Function | Description |
|----------|-------------|
| [`getDeviceInfo()`](/api/device-info) | Device identification and hardware info |
| [`getBatteryInfo()`](/api/battery-info) | Battery status and health |
| [`getNetworkInfo()`](/api/network-info) | Network connection details |
| [`getStorageInfo()`](/api/storage-info) | Storage capacity information |
| [`getDisplayInfo()`](/api/display-info) | Display properties and capabilities |

## Quick Reference

```typescript
import { 
  getDeviceInfo,    // → DeviceInfoResponse
  getBatteryInfo,   // → BatteryInfo
  getNetworkInfo,   // → NetworkInfo
  getStorageInfo,   // → StorageInfo
  getDisplayInfo    // → DisplayInfo
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
  DisplayInfo 
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


