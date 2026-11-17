# Adaptive Apps

Raw readings tell you *what* the device is doing. These two APIs tell you *what
to do about it*: one signal for how much work the app should do **right now**,
and one for how capable the hardware is **in general**.

| Function | Returns | Answers |
|----------|---------|---------|
| `getPowerProfile()` | `PowerProfile` | "Should I hold back right now?" → `full` / `balanced` / `saver` |
| `watchPowerProfile()` | `Promise<UnwatchFn>` | Same, pushed to you whenever it changes |
| `getDeviceTier()` | `DeviceTier` | "How much can this device handle?" → `low` / `mid` / `high` |

## Power profile

```typescript
import { getPowerProfile, watchPowerProfile } from 'tauri-plugin-device-info-api';

const { profile, reasons } = await getPowerProfile();
// profile: 'saver', reasons: ['lowPowerMode', 'batteryLow']

const unwatch = await watchPowerProfile(({ profile }) => {
  document.documentElement.dataset.power = profile;
});
```

```css
/* Turn off expensive effects when the device is under pressure */
[data-power="saver"] * { animation: none !important; transition: none !important; }
[data-power="saver"] .glass { backdrop-filter: none; }
```

### How the profile is chosen

Every active condition implies a profile; the **most restrictive one wins**, and
all active conditions are listed in `reasons`.

| Reason | Condition | Implies |
|--------|-----------|---------|
| `lowPowerMode` | OS power saving is on | `saver` |
| `thermalCritical` | Thermal state `critical` | `saver` |
| `thermalSerious` | Thermal state `serious` | `saver` |
| `thermalFair` | Thermal state `fair` | `balanced` |
| `batteryCritical` | On battery, ≤ 20% | `saver` |
| `batteryLow` | On battery, ≤ 40% | `balanced` |

Battery reasons apply only **on battery**: not charging *and* not on external
power. A laptop on AC that is full, held at a charge limit or "not charging"
is on external power (`onExternalPower: true`), so it never gets a battery
reason. When a platform can't tell, the device counts as on battery. A device
without a battery (most desktops), or one reporting an unknown or out-of-range
level (e.g. the iOS Simulator), gets no battery reason either.

### Response

```json
{
  "profile": "balanced",
  "reasons": ["batteryLow"],
  "batteryLevel": 34,
  "isCharging": false,
  "lowPowerMode": false,
  "thermalState": "nominal",
  "onExternalPower": false
}
```

| Field | Type | Description |
|-------|------|-------------|
| `profile` | `'full' \| 'balanced' \| 'saver'` | The recommendation |
| `reasons` | `PowerProfileReason[]` | Why it isn't `full` (empty when it is) |
| `batteryLevel` | `number?` | 0–100; absent without a battery |
| `isCharging` | `boolean?` | |
| `lowPowerMode` | `boolean?` | Absent when the OS doesn't expose it |
| `thermalState` | `'nominal' \| 'fair' \| 'serious' \| 'critical'?` | Absent on Windows |
| `onExternalPower` | `boolean?` | On an AC / USB-C adapter, even when not charging; absent when the OS doesn't say |

The raw fields are included so you can apply your own policy if the defaults
don't fit.

### Platform sources

| Platform | Low power mode | Thermal state | External power |
|----------|----------------|---------------|----------------|
| macOS | `NSProcessInfo.isLowPowerModeEnabled` (macOS 12+) | `NSProcessInfo.thermalState` | IOKit providing power source = AC |
| iOS | `ProcessInfo.isLowPowerModeEnabled` | `ProcessInfo.thermalState` | `UIDevice.batteryState` charging / full |
| Android | `PowerManager.isPowerSaveMode` | `PowerManager.currentThermalStatus` (Android 10+) | Battery intent `EXTRA_PLUGGED` |
| Windows | Battery / Energy Saver (`GetSystemPowerStatus`) | — | `GetSystemPowerStatus` `ACLineStatus` |
| Linux | ACPI platform profile = `low-power` | CPU/SoC `/sys/class/thermal` zones, headroom to critical trip | Any online non-battery `/sys/class/power_supply` |

Android's seven thermal levels are mapped onto four: `LIGHT`/`MODERATE` → `fair`,
`SEVERE` → `serious`, `CRITICAL` and above → `critical`. On Linux, a zone within
25 °C / 15 °C / 5 °C of its critical trip point is `fair` / `serious` /
`critical`. Only CPU/SoC zones count (`x86_pkg_temp`, `coretemp`, `k10temp`,
`acpitz`, `cpu*`, `soc*`, ...), so battery, skin and Wi-Fi sensors with low trip
points don't make a cool laptop read as hot; trip points below 80 °C are
replaced by a 100 °C default.

`watchPowerProfile` polls every 5 s by default (minimum 1 s); pass
`{ intervalMs }` to change it.

## Device tier

```typescript
import { getDeviceTier } from 'tauri-plugin-device-info-api';

const { tier, score } = await getDeviceTier();
document.documentElement.dataset.tier = tier;

const particleCount = { low: 0, mid: 200, high: 1000 }[tier];
```

### How the tier is scored

The score (0–100) is a weighted average of memory, logical cores and peak CPU
clock. Each signal is placed on a log scale between a floor and a ceiling for
the device's **form factor**, so a phone is compared with phones and a laptop
with laptops:

| Signal | Desktop range | Mobile range | Weight (desktop / mobile) |
|--------|---------------|--------------|---------------------------|
| Memory | 4 – 32 GB | 2 – 12 GB | 0.50 / 0.55 |
| Logical cores | 2 – 16 | 4 – 8 | 0.30 / 0.15 |
| Peak CPU clock | 1.5 – 4.5 GHz | 1.4 – 3.2 GHz | 0.20 / 0.30 |

Signals the platform doesn't report (e.g. CPU clock on iOS and Apple Silicon)
are left out and the other weights rescaled.

| Score | Tier |
|-------|------|
| 0 – 29 | `low` |
| 30 – 64 | `mid` |
| 65 – 100 | `high` |

On Android, OS hints override the score: a device flagged
`isLowRamDevice` is always `low`, and a media performance class of 31+ / 33+
guarantees at least `mid` / `high`.

Example classifications:

| Device | Score | Tier |
|--------|-------|------|
| 4 GB dual-core laptop | 3 | `low` |
| M1 MacBook Air, 8 GB | 46 | `mid` |
| M3 Pro, 18 GB | 78 | `high` |
| iPhone 12 (4 GB) | 43 | `mid` |
| iPhone 15 Pro (8 GB) | 73 | `high` |

### Response

```json
{
  "tier": "high",
  "score": 90,
  "hardware": {
    "cpuCores": 15,
    "physicalCores": 15,
    "cpuModel": "Apple M5 Pro",
    "totalMemory": 25769803776,
    "isMobile": false
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `tier` | `'low' \| 'mid' \| 'high'` | Capability class |
| `score` | `number` | 0–100, for finer-grained decisions than the tier |
| `hardware.cpuCores` | `number?` | Logical cores |
| `hardware.physicalCores` | `number?` | Physical cores (desktop) |
| `hardware.cpuModel` | `string?` | CPU / SoC name or model identifier |
| `hardware.cpuMaxFrequencyMhz` | `number?` | Peak clock, where available |
| `hardware.totalMemory` | `number?` | Bytes |
| `hardware.isMobile` | `boolean` | Which scale was used |
| `hardware.isLowRamDevice` | `boolean?` | Android only |
| `hardware.mediaPerformanceClass` | `number?` | Android 12+ only |

The tier reflects hardware, which doesn't change at runtime, so there's no
`watch` variant. Combine it with the power profile for decisions that depend on
both:

```typescript
const [{ tier }, { profile }] = await Promise.all([getDeviceTier(), getPowerProfile()]);
const quality = profile === 'saver' ? 'low' : tier;
```

## Rust API

Both are also available from Rust, and the scoring functions are public:

```rust
use tauri_plugin_device_info::DeviceInfoExt;

let profile = app.device_info().get_power_profile()?;
let tier = app.device_info().get_device_tier()?;

// Pure functions, for your own inputs or tests:
tauri_plugin_device_info::adaptive::power_profile(&battery, &power_state);
tauri_plugin_device_info::adaptive::device_tier(hardware);
```
