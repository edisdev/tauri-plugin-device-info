# Reactive Watch API

Instead of calling a getter on a timer, you can **subscribe** to a kind of
device information and receive a callback whenever the value changes.

Each `watch*` function delivers the **current value immediately**, then invokes
your callback again on every subsequent change. It returns an async `unwatch`
function that stops watching and removes the listener.

## Functions

| Function | Delivers | Watches for |
|----------|----------|-------------|
| `watchBattery()` | `BatteryInfo` | Charge level, charging state, health |
| `watchNetwork()` | `NetworkInfo` | Connectivity changes (Wi-Fi ↔ offline, IP, MAC) |
| `watchStorage()` | `StorageInfo` | Total / free space |
| `watchDisplay()` | `DisplayInfo` | Resolution, scale, refresh rate (e.g. plugging in a monitor) |
| `watchDevice()` | `DeviceInfoResponse` | Device identity fields (rarely changes) |

## Signatures

```typescript
function watchBattery(callback: (info: BatteryInfo) => void, options?: WatchOptions): Promise<UnwatchFn>
function watchNetwork(callback: (info: NetworkInfo) => void, options?: WatchOptions): Promise<UnwatchFn>
function watchStorage(callback: (info: StorageInfo) => void, options?: WatchOptions): Promise<UnwatchFn>
function watchDisplay(callback: (info: DisplayInfo) => void, options?: WatchOptions): Promise<UnwatchFn>
function watchDevice(callback: (info: DeviceInfoResponse) => void, options?: WatchOptions): Promise<UnwatchFn>
```

## Options & return type

```typescript
interface WatchOptions {
  /**
   * Polling interval in milliseconds, used only for kinds that fall back to
   * polling (native event-driven kinds ignore it). Only the first subscriber
   * for a given kind sets the interval.
   */
  intervalMs?: number;
}

/** Stops a watcher and removes its listener. Returned by every `watch*` function. */
type UnwatchFn = () => Promise<void>;
```

## Basic usage

```typescript
import { watchBattery } from 'tauri-plugin-device-info-api';

// Called immediately with the current value, then on every change.
const unwatch = await watchBattery((battery) => {
  console.log(`Battery: ${battery.level}% (charging: ${battery.isCharging})`);
});

// Later, when you no longer need updates:
await unwatch();
```

### Cleaning up multiple watchers

Always stop watchers when the component/view is torn down, otherwise the
backend monitor stays alive.

```typescript
import { watchBattery, watchNetwork, watchDisplay } from 'tauri-plugin-device-info-api';
import type { UnwatchFn } from 'tauri-plugin-device-info-api';

const unwatchers: UnwatchFn[] = [];

unwatchers.push(await watchBattery((info) => { /* ... */ }));
unwatchers.push(await watchNetwork((info) => { /* ... */ }));
unwatchers.push(await watchDisplay((info) => { /* ... */ }));

// On teardown:
await Promise.all(unwatchers.map((unwatch) => unwatch()));
```

## How updates are detected

The public API and the emitted events are identical across platforms, but the
engine behind each kind differs:

- Where the OS exposes a **native, event-driven** source, updates are delivered
  the instant the OS reports a change and the CPU stays idle in between.
- Otherwise a **change-detecting poller** reads the value on an interval and
  emits only when it actually changed.

| Kind | macOS | Windows / Linux |
|------|-------|-----------------|
| `battery` | Native — IOKit power notifications | Polled |
| `display` | Native — Core Graphics reconfiguration callback | Polled |
| `network` | Native — SystemConfiguration reachability | Polled |
| `storage` | Polled (no OS change event) | Polled |
| `device` | Polled (rarely changes) | Polled |

### Polling intervals

`intervalMs` only applies to polled kinds. Defaults and minimums are **per
kind**, because some getters are expensive and some values rarely change:

| Kind | Default | Minimum |
|------|---------|---------|
| `device` | 60000 ms | 10000 ms |
| `storage` | 10000 ms | 1000 ms |
| others (when polled) | 2000 ms | 250 ms |

```typescript
// Poll storage a little more eagerly (still floored at 1000 ms):
const unwatch = await watchStorage((info) => console.log(info.freeSpace), {
  intervalMs: 2000,
});
```

::: tip Reference counting
Subscribers to the same kind share a single monitor. It starts on the first
subscriber and is torn down once the last one calls `unwatch()`. Only the first
subscriber's `intervalMs` takes effect.
:::

## Permissions

The watch API requires the `start-watching` and `stop-watching` commands.
`device-info:default` already includes them; to grant them individually:

```json
{
  "permissions": [
    "device-info:allow-start-watching",
    "device-info:allow-stop-watching"
  ]
}
```

## Example output

`watchBattery` delivers the same shape as `getBatteryInfo`:

```json
{
  "level": 85,
  "isCharging": true,
  "health": "Good"
}
```

## Notes

- The current value is always delivered once, immediately, so you don't need a
  separate `getBatteryInfo()` call to seed initial state.
- `unwatch()` is async — `await` it if you need the backend monitor fully torn
  down before continuing.
- On a kind with a native source (e.g. `battery` on macOS), `intervalMs` is
  ignored.
