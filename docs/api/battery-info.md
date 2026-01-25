# getBatteryInfo()

Returns battery status and health information.

## Signature

```typescript
function getBatteryInfo(): Promise<BatteryInfo>
```

## Response Type

```typescript
interface BatteryInfo {
  level?: number;
  isCharging?: boolean;
  health?: string;
}
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `level` | `number?` | Current battery charge level (0-100 percentage) |
| `isCharging` | `boolean?` | Whether the device is currently connected to power and charging |
| `health` | `string?` | Battery health status (e.g., "Good", "Fair", "Poor") |

## Example

```typescript
import { getBatteryInfo } from 'tauri-plugin-device-info-api';

const battery = await getBatteryInfo();

console.log(`Battery Level: ${battery.level}%`);
console.log(`Charging: ${battery.isCharging ? 'Yes' : 'No'}`);
console.log(`Health: ${battery.health}`);

// Visual indicator
if (battery.level < 20) {
  console.warn('Low battery!');
}
```

## Platform-specific Behavior

### Windows
- Uses `battery` crate
- Health calculated from state of health percentage

### macOS
- Uses `pmset -g batt` command
- Health may be "Good", "Good (Charging)", or "Good (Full)"

### Linux
- Uses `battery` crate
- Reads from `/sys/class/power_supply/`

### iOS
- Uses `UIDevice.current.batteryLevel`
- Requires `isBatteryMonitoringEnabled = true`
- Health derived from battery state

### Android
- Uses `BatteryManager` via Intent broadcast
- Full access to level, charging, and health

## Example Output

```json
{
  "level": 85,
  "isCharging": true,
  "health": "Good"
}
```

## Notes

- On desktop systems without a battery, fields may be `null`
- Battery level is rounded to nearest integer on some platforms
- Health status strings vary by platform
