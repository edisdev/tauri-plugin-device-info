# Getting Started

## Installation

### Using Tauri CLI (Recommended)

```bash
yarn tauri add device-info
```

### Manual Installation

**1. Add the Rust dependency:**

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-device-info = "1.0"  # See https://crates.io/crates/tauri-plugin-device-info for latest
```

**2. Add the JavaScript package:**

```bash
yarn add tauri-plugin-device-info-api
```

## Setup

Register the plugin in your Tauri app:

```rust
// src-tauri/src/lib.rs (Tauri v2)
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_device_info::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Permissions

Add the required permissions to your capabilities:

```json
// src-tauri/capabilities/default.json
{
  "permissions": ["device-info:default"]
}
```

Or add individual permissions:

- `device-info:allow-get-device-info`
- `device-info:allow-get-battery-info`
- `device-info:allow-get-network-info`
- `device-info:allow-get-storage-info`
- `device-info:allow-get-display-info`
- `device-info:allow-start-watching`
- `device-info:allow-stop-watching`

## Basic Usage

```typescript
import {
  getDeviceInfo,
  getBatteryInfo,
  getNetworkInfo,
  getStorageInfo,
  getDisplayInfo,
} from "tauri-plugin-device-info-api";

async function loadDeviceInfo() {
  // Device information
  const device = await getDeviceInfo();
  console.log(`Device: ${device.model}`);
  console.log(`Manufacturer: ${device.manufacturer}`);

  // Battery status
  const battery = await getBatteryInfo();
  console.log(`Battery: ${battery.level}%`);
  console.log(`Charging: ${battery.isCharging}`);

  // Network information
  const network = await getNetworkInfo();
  console.log(`IP: ${network.ipAddress}`);
  console.log(`Type: ${network.networkType}`);

  // Storage information
  const storage = await getStorageInfo();
  console.log(`Total: ${storage.totalSpace} bytes`);
  console.log(`Free: ${storage.freeSpace} bytes`);

  // Display information
  const display = await getDisplayInfo();
  console.log(`Resolution: ${display.width}x${display.height}`);
  console.log(`Refresh Rate: ${display.refreshRate}Hz`);
}
```

## Reactive Updates

Instead of calling a getter on a timer, subscribe to a kind and get a callback
whenever the value changes. The current value is delivered immediately, then on
every change. On macOS these are native and event-driven; elsewhere they fall
back to change-detecting polling.

```typescript
import { watchBattery } from "tauri-plugin-device-info-api";

const unwatch = await watchBattery((battery) => {
  console.log(`Battery: ${battery.level}% (charging: ${battery.isCharging})`);
});

// Stop watching when you're done (e.g. on component teardown):
await unwatch();
```

See the [Reactive Watch API](/api/watch) for all `watch*` functions and options.

## TypeScript Types

All types are exported and can be imported:

```typescript
import type {
  DeviceInfoResponse,
  BatteryInfo,
  NetworkInfo,
  StorageInfo,
  DisplayInfo,
  WatchOptions,
  UnwatchFn,
} from "tauri-plugin-device-info-api";
```

## Next Steps

- Check out the [API Reference](/api/) for detailed documentation
- See [Platform-specific notes](/platforms/) for each operating system
- View [Examples](/examples) for real-world usage patterns
