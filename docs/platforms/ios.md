# iOS

![iOS Device Info Preview](/DeviceInfo_iOS.png){width=300}

## Implementation

iOS support uses native Swift with UIKit and Network frameworks.

## Technologies Used

- **UIKit** - Device and display information
- **Network.framework** - Network monitoring
- **Security.framework** - Keychain for persistent UUID

## APIs Used

| Information | API |
|-------------|-----|
| Device Name | `UIDevice.current.name` |
| Model | `utsname().machine` |
| UUID | `UIDevice.identifierForVendor` |
| Battery Level | `UIDevice.current.batteryLevel` |
| Network Type | `NWPathMonitor` |
| Display | `UIScreen.main` |

## Privacy Restrictions

### Device Name

iOS 16+ restricts access to the real device name. Without the entitlement, iOS returns generic names like "iPhone".

The plugin falls back to hostname for a more descriptive name.

### MAC Address

Apple has blocked MAC address access since iOS 7. The plugin returns "unavailable".

### Serial Number

The real serial number is not accessible. The plugin stores a persistent UUID in Keychain as a substitute.

## Network Interfaces

| Interface | Description |
|-----------|-------------|
| `en0` | WiFi |
| `pdp_ip0` | Cellular data |

The plugin checks both interfaces for IP address.

## Entitlements

For full device name access, add to your entitlements:

```xml
<key>com.apple.developer.device-information.user-assigned-device-name</key>
<true/>
```

Note: This entitlement requires Apple approval.

## Example Output

```json
{
  "deviceName": "Hatice's iPhone",
  "manufacturer": "Apple Inc.",
  "model": "iPhone14,2",
  "uuid": "12345678-1234-5678-9ABC-DEF012345678",
  "serial": "A1B2C3D4-E5F6-7890-ABCD-EF1234567890"
}
```
