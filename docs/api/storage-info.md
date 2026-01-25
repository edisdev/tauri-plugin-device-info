# getStorageInfo()

Returns storage capacity and type information.

## Signature

```typescript
function getStorageInfo(): Promise<StorageInfo>
```

## Response Type

```typescript
interface StorageInfo {
  totalSpace: number;
  freeSpace: number;
  storageType?: string;
}
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `totalSpace` | `number` | Total storage capacity in bytes |
| `freeSpace` | `number` | Available (free) storage space in bytes |
| `storageType` | `string?` | Storage technology type: "Ssd", "Hdd", "Removable", "Unknown" |

## Example

```typescript
import { getStorageInfo } from 'tauri-plugin-device-info-api';

const storage = await getStorageInfo();

// Format bytes to human-readable
function formatBytes(bytes: number): string {
  const gb = bytes / (1024 ** 3);
  return `${gb.toFixed(1)} GB`;
}

console.log(`Total: ${formatBytes(storage.totalSpace)}`);
console.log(`Free: ${formatBytes(storage.freeSpace)}`);
console.log(`Used: ${formatBytes(storage.totalSpace - storage.freeSpace)}`);
console.log(`Type: ${storage.storageType}`);

// Calculate usage percentage
const usedPercent = ((storage.totalSpace - storage.freeSpace) / storage.totalSpace * 100).toFixed(1);
console.log(`Usage: ${usedPercent}%`);
```

## Platform-specific Behavior

### Windows
- Uses `sysinfo` crate
- Root disk is `C:\`
- Storage type detection via disk kind

### macOS
- Uses `sysinfo` crate
- Root disk is `/`
- Accurate SSD/HDD detection

### Linux
- Uses `sysinfo` crate
- Root disk is `/`
- May show "Unknown" for some disk types

### iOS
- Uses `FileManager` and `URL.resourceValues`
- Reports internal storage only
- Storage type is always "internal"

### Android
- Uses `Environment.getExternalStorageDirectory()`
- Reports primary storage volume
- Storage type is "internal" or "external"

## Example Output

```json
{
  "totalSpace": 499963174912,
  "freeSpace": 123456789012,
  "storageType": "Ssd"
}
```

## Notes

- Values are in bytes (not KB, MB, or GB)
- Division by `1024^3` gives gigabytes
- External/removable storage not included in total
