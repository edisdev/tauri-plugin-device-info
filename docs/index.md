---
layout: home

hero:
  name: Tauri Plugin Device Info
  text: Cross-platform device information
  tagline: Get device, battery, network, storage, and display info on all platforms
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/edisdev/tauri-plugin-device-info

features:
  - icon: 📱
    title: Device Info
    details: UUID, manufacturer, model, serial number, and device name
  - icon: 🔋
    title: Battery Info
    details: Charge level, charging status, and battery health
  - icon: 🌐
    title: Network Info
    details: IP address, network type, and MAC address
  - icon: 💾
    title: Storage Info
    details: Total space, free space, and storage type (SSD/HDD)
  - icon: 🖥️
    title: Display Info
    details: Resolution, scale factor, and refresh rate
  - icon: 🔔
    title: Reactive Watch
    details: Subscribe to changes — native event-driven on macOS, polling fallback elsewhere
  - icon: ⚡
    title: Adaptive Apps
    details: One full / balanced / saver power signal and a low / mid / high device tier, so your app adjusts itself
  - icon: 🌍
    title: Cross-platform
    details: Works on Windows, macOS, Linux, iOS, and Android
---

## Preview

<div align="center">
  <img src="/DeviceInfo_Promo.gif" alt="The plugin in the example app: live signals on macOS and iOS, then the power profile adapting on Windows and Android" width="900" style="border-radius: 8px; margin-bottom: 20px;">
  <div style="display: flex; gap: 16px; justify-content: center; align-items: flex-start; flex-wrap: wrap;">
    <img src="/DeviceInfo_macOS.gif" alt="Vitals on macOS: switching pages, simulating the Saver profile and changing themes" width="560" style="border-radius: 8px;">
    <img src="/DeviceInfo_iOS.gif" alt="Vitals on iPhone: live battery and thermal readings, the Adaptive page and themes" width="230" style="border-radius: 8px;">
  </div>
  <p><i><b>Vitals</b> — the example app built on this plugin. Real readings on macOS, iOS, Windows 10 and Android 16.</i></p>
</div>

## Platform Support

| Platform | Status |
|----------|--------|
| Windows  | ✅ Full Support |
| macOS    | ✅ Full Support |
| Linux    | ✅ Full Support |
| iOS      | ✅ Full Support |
| Android  | ✅ Full Support |

## Quick Example

```typescript
import { 
  getDeviceInfo, 
  getBatteryInfo,
  getNetworkInfo,
  getStorageInfo,
  getDisplayInfo
} from 'tauri-plugin-device-info-api';

// Get device information
const device = await getDeviceInfo();
console.log(device.model);  // "MacBook Pro"

// Get battery status
const battery = await getBatteryInfo();
console.log(`Battery: ${battery.level}%`);

// Get network info
const network = await getNetworkInfo();
console.log(`IP: ${network.ipAddress}`);
```

Prefer push over polling? Subscribe to changes with the
[reactive watch API](/api/watch):

```typescript
import { watchBattery } from 'tauri-plugin-device-info-api';

const unwatch = await watchBattery((b) => console.log(`Battery: ${b.level}%`));
// await unwatch(); // stop when done
```
