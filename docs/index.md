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
  - icon: 🌍
    title: Cross-platform
    details: Works on Windows, macOS, Linux, iOS, and Android
---

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
