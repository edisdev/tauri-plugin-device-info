# getNetworkInfo()

Returns network connection details and identifiers.

## Signature

```typescript
function getNetworkInfo(): Promise<NetworkInfo>
```

## Response Type

```typescript
interface NetworkInfo {
  ipAddress?: string;
  networkType?: string;
  macAddress?: string;
}
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `ipAddress` | `string?` | Local IPv4 address (e.g., "192.168.1.100") |
| `networkType` | `string?` | Connection type: "wifi", "ethernet", "cellular", or "unknown" |
| `macAddress` | `string?` | MAC address (unavailable on iOS/Android due to privacy) |

## Example

```typescript
import { getNetworkInfo } from 'tauri-plugin-device-info-api';

const network = await getNetworkInfo();

console.log(`IP Address: ${network.ipAddress}`);
console.log(`Network Type: ${network.networkType}`);
console.log(`MAC Address: ${network.macAddress}`);
```

## Platform-specific Behavior

### Windows / macOS / Linux
- Uses `local-ip-address` and `default-net` crates
- MAC address from the default gateway interface
- Network type from interface name

### iOS
- Uses `NWPathMonitor` for network type detection
- IP address from `getifaddrs()` (en0 for WiFi, pdp_ip0 for cellular)
- MAC address returns "unavailable" (Apple privacy restriction)

### Android
- Uses `WifiManager` for WiFi info
- MAC address restricted on Android 6.0+ (returns "02:00:00:00:00:00")

## Example Output

```json
{
  "ipAddress": "192.168.1.105",
  "networkType": "wifi",
  "macAddress": "AA:BB:CC:DD:EE:FF"
}
```

## Notes

- MAC address is privacy-restricted on mobile platforms
- IPv6 addresses are not currently returned
- VPN connections may affect the reported IP
