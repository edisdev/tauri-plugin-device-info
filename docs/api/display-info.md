# getDisplayInfo()

Returns display/screen properties and capabilities.

## Signature

```typescript
function getDisplayInfo(): Promise<DisplayInfo>
```

## Response Type

```typescript
interface DisplayInfo {
  width: number;
  height: number;
  scaleFactor: number;
  refreshRate?: number;
}
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `width` | `number` | Screen width in physical pixels |
| `height` | `number` | Screen height in physical pixels |
| `scaleFactor` | `number` | Display scale factor (e.g., 2.0 for Retina/HiDPI) |
| `refreshRate` | `number?` | Screen refresh rate in Hz (e.g., 60.0, 120.0, 144.0) |

## Example

```typescript
import { getDisplayInfo } from 'tauri-plugin-device-info-api';

const display = await getDisplayInfo();

console.log(`Resolution: ${display.width}x${display.height}`);
console.log(`Scale Factor: ${display.scaleFactor}x`);
console.log(`Refresh Rate: ${display.refreshRate}Hz`);

// Calculate logical resolution
const logicalWidth = display.width / display.scaleFactor;
const logicalHeight = display.height / display.scaleFactor;
console.log(`Logical: ${logicalWidth}x${logicalHeight}`);
```

## Platform-specific Behavior

### Windows
- Uses `Win32_VideoController` via WMI
- Refresh rate from `CurrentRefreshRate`


### macOS
- Uses native CoreGraphics API (`CGDisplay`)
- Fast and accurate refresh rate detection
- Returns `null` for ProMotion displays with variable refresh rate

### Linux
- Uses `xrandr --current` command
- Requires X11 (Wayland support limited)
- Parses current mode from output

### iOS
- Uses `UIScreen.main`
- Physical pixels = bounds × scale
- Refresh rate from `maximumFramesPerSecond`

### Android
- Uses `WindowManager.getDefaultDisplay()`
- Refresh rate from `Display.getRefreshRate()`

## Example Output

```json
{
  "width": 2560,
  "height": 1600,
  "scaleFactor": 2.0,
  "refreshRate": 60.0
}
```

## Notes

- Width and height are in **physical pixels**
- Divide by `scaleFactor` for logical/CSS pixels
- Refresh rate may be `null` on variable refresh rate displays
- Multi-monitor setups report the primary display
