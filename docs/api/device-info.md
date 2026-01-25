# getDeviceInfo()

Returns comprehensive device identification and hardware information.

## Signature

```typescript
function getDeviceInfo(): Promise<DeviceInfoResponse>
```

## Response Type

```typescript
interface DeviceInfoResponse {
  uuid?: string;
  manufacturer?: string;
  model?: string;
  serial?: string;
  android_id?: string;
  device_name?: string;
}
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `uuid` | `string?` | Unique device identifier (Hardware UUID or GUID) |
| `manufacturer` | `string?` | Device manufacturer (e.g., "Apple Inc.", "Dell Inc.") |
| `model` | `string?` | Device model name (e.g., "MacBook Pro", "OptiPlex 7090") |
| `serial` | `string?` | Device serial number (may be restricted on some platforms) |
| `android_id` | `string?` | Android-specific device ID (Android only) |
| `device_name` | `string?` | User-assigned device name (hostname) |

## Example

```typescript
import { getDeviceInfo } from 'tauri-plugin-device-info-api';

const device = await getDeviceInfo();

console.log(`Device: ${device.deviceName}`);
console.log(`Model: ${device.model}`);
console.log(`Manufacturer: ${device.manufacturer}`);
console.log(`UUID: ${device.uuid}`);
console.log(`Serial: ${device.serial}`);
```

## Platform-specific Behavior

### Windows
- Uses WMI (Windows Management Instrumentation)
- UUID from `Win32_ComputerSystemProduct`
- Serial from `Win32_BIOS` or `Win32_ComputerSystemProduct`

### macOS
- Uses `system_profiler SPHardwareDataType`
- Manufacturer is always "Apple Inc."
- Full access to UUID and serial

### Linux
- Reads from `/sys/class/dmi/id/`
- UUID fallback to `/etc/machine-id`
- May require root for serial number

### iOS
- UUID from `identifierForVendor` (vendor-specific)
- Serial uses persistent Keychain-stored UUID
- Real device name may require entitlement

### Android
- UUID from `Settings.Secure.ANDROID_ID`
- Model and manufacturer from `android.os.Build`

## Example Output

```json
{
  "deviceName": "MacBook-Pro",
  "manufacturer": "Apple Inc.",
  "model": "MacBook Pro",
  "uuid": "12345678-1234-5678-9ABC-DEF012345678",
  "serial": "C02ABC123DEF",
  "android_id": null
}
```
