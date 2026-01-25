# Platform Support

This plugin provides full support for all major desktop and mobile platforms.

## Support Matrix

| Platform | Device | Battery | Network | Storage | Display |
|----------|--------|---------|---------|---------|---------|
| Windows  | ✅ | ✅ | ✅ | ✅ | ✅ |
| macOS    | ✅ | ✅ | ✅ | ✅ | ✅ |
| Linux    | ✅ | ✅ | ✅ | ✅ | ✅ |
| iOS      | ✅ | ✅ | ✅ | ✅ | ✅ |
| Android  | ✅ | ✅ | ✅ | ✅ | ✅ |

## Platform-Specific Notes

### Desktop

| Platform | Technology | Notes |
|----------|------------|-------|
| [Windows](/platforms/windows) | WMI | Full access to all hardware info |
| [macOS](/platforms/macos) | system_profiler, CoreGraphics | Full access, CoreGraphics for display |
| [Linux](/platforms/linux) | sysfs, procfs | May require root for some info |

### Mobile

| Platform | Technology | Notes |
|----------|------------|-------|
| [iOS](/platforms/ios) | UIKit, Network.framework | Privacy restrictions on MAC address |
| [Android](/platforms/android) | Native APIs | MAC restricted on Android 6.0+ |

## Privacy Considerations

Some information may be restricted for privacy reasons:

| Information | Desktop | iOS | Android |
|-------------|---------|-----|---------|
| Device Name | ✅ | ⚠️ Entitlement needed | ✅ |
| Serial Number | ✅ | ⚠️ Keychain UUID | ⚠️ Restricted |
| MAC Address | ✅ | ❌ Unavailable | ❌ Restricted |

## Implementation Details

Each platform has its own implementation:

```
src/
├── desktop.rs          # Desktop coordinator
│   ├── desktop/
│   │   ├── windows.rs  # WMI queries
│   │   ├── macos.rs    # system_profiler + CoreGraphics
│   │   └── linux.rs    # sysfs + procfs
├── mobile.rs           # Mobile coordinator
├── ios/                # Swift implementation
└── android/            # Kotlin implementation
```
