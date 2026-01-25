# Android

<div style="display: flex; gap: 10px; flex-wrap: wrap;">
  <img src="/DeviceInfo_Android.png" width="300" style="border-radius: 8px;">
  <img src="/DeviceInfo_Android_Gif.gif" width="300" style="border-radius: 8px;">
</div>

## Implementation

Android support uses native Kotlin with Android SDK APIs.

## Technologies Used

- **android.os.Build** - Device information
- **BatteryManager** - Battery status
- **WifiManager** - Network information
- **WindowManager** - Display properties
- **Environment** - Storage information

## APIs Used

| Information | API |
|-------------|-----|
| Device Name | `Settings.Global.DEVICE_NAME` |
| Model | `Build.MODEL` |
| Manufacturer | `Build.MANUFACTURER` |
| UUID | `Settings.Secure.ANDROID_ID` |
| Battery | `BatteryManager` |
| Network | `WifiManager`, `TelephonyManager` |
| Display | `WindowManager.getDefaultDisplay()` |
| Storage | `Environment.getExternalStorageDirectory()` |

## Permissions

No special permissions are required for basic device info. Optional permissions:

```xml
<!-- For more network details -->
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
```

## Privacy Restrictions

### ANDROID_ID

`ANDROID_ID` is unique per app signing key and device. It changes if:
- User factory resets the device
- App is signed with a different key

### MAC Address

Since Android 6.0 (API 23), `WifiInfo.getMacAddress()` returns a constant fake value `02:00:00:00:00:00`.

The plugin returns "restricted" or the constant value.

## Example Output

```json
{
  "deviceName": "Pixel 7",
  "manufacturer": "Google",
  "model": "Pixel 7",
  "uuid": "abc123def456789",
  "serial": null,
  "android_id": "abc123def456789"
}
```

## Battery Health Values

Android provides detailed battery health:

| Value | Meaning |
|-------|---------|
| `BATTERY_HEALTH_GOOD` | Good |
| `BATTERY_HEALTH_OVERHEAT` | Overheating |
| `BATTERY_HEALTH_DEAD` | Dead |
| `BATTERY_HEALTH_COLD` | Cold |
| `BATTERY_HEALTH_UNKNOWN` | Unknown |

## Minimum SDK

The plugin requires:
- **minSdkVersion**: 21 (Android 5.0)
- **targetSdkVersion**: 33+ recommended
