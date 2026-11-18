# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.1] - 2026-11-19

### Fixed

- **MAC address could change after upgrading to 1.3.0** (Windows, macOS,
  Linux). 1.3.0 picked the "default" network interface through a route to
  `10.254.254.254` (the `netdev` crate's method) instead of `1.1.1.1:80`. On
  machines with a separate route to 10.0.0.0/8, such as a second network card
  for a POS network, a split-tunnel VPN, or some Docker or Hyper-V networks, it
  could pick a different card and report a different MAC than 1.2.x. The
  interface is chosen through `1.1.1.1:80` again, exactly as before 1.3.0. If
  you derive an identity from the MAC address, skip 1.3.0.

## [1.3.0] - 2026-11-18

### Changed

- **Minimum versions:** the plugin now requires Tauri **2.12** (`tauri`
  crate) and `@tauri-apps/api` **^2.12.5**.
- **Dependencies:**
  - `sysinfo` 0.40, `wmi` 0.18 (Windows), and the other dependencies
    updated within range.
  - Apple Silicon: `sysinfo` now reads the CPU clock, so the device tier
    score can rise slightly (an M5 Pro moves from 90 to 92).
- **Mobile builds:** desktop-only crates (`sysinfo`, `local-ip-address`, the
  network crate) are no longer compiled for iOS and Android
  ([#7](https://github.com/edisdev/tauri-plugin-device-info/pull/7)).
- **Package manager:** the repository uses pnpm instead of yarn. Install
  commands in the docs show npm, pnpm and yarn.

### Fixed

- **Unmaintained crates replaced:**
  - `battery` is replaced by `starship-battery`, its maintained fork, on
    Windows and Linux. On macOS it was never used and is no longer compiled.
  - `default-net` is replaced by `netdev`, its successor.
- **Security advisories:** no known advisories remain in the JS
  dependencies of the plugin, the docs or the example app.

## [1.2.0] - 2026-11-17

### Added

- **Adaptive Apps** — signals that turn raw readings into decisions.
  - `getPowerProfile()` / `watchPowerProfile()` — one `full | balanced | saver`
    recommendation combining battery level, charging state, OS power-saving mode
    (Low Power Mode, Battery Saver, Linux `low-power` profile) and thermal
    pressure, plus the `reasons` behind it. Battery thresholds apply only on
    battery — not while charging or on external power (`onExternalPower`:
    AC/USB-C adapter detected on macOS, Windows, Linux, iOS and Android, so a
    laptop on AC that is full or held at a charge limit is not "low") — and an
    unknown battery level (iOS Simulator, unsupported Android capacity) is
    ignored rather than read as empty. On Linux only CPU/SoC thermal zones are
    considered.
  - `getDeviceTier()` — a `low | mid | high` capability class and 0-100 score
    from memory, CPU cores and peak clock, scored against the device's form
    factor; honours Android's low-RAM flag and media performance class.
  - `get_power_profile` / `get_device_tier` commands with matching
    `allow-get-power-profile` / `allow-get-device-tier` permissions, included in
    `device-info:default`.
  - New `power` watch kind (polled, default 5 s).
  - Rust: `get_power_profile()`, `get_device_tier()`, `get_power_state()` and
    `get_hardware_info()` on `DeviceInfo`, and the pure
    `adaptive::power_profile` / `adaptive::device_tier` functions.
  - New types: `PowerProfile`, `PowerProfileKind`, `PowerProfileReason`,
    `ThermalState`, `DeviceTier`, `DeviceTierKind`, `HardwareInfo`.

### Fixed

- **Watch API: late subscribers now get the current value.** A `watch*` call
  for a kind that was already being watched (a second component, or the same
  page after a webview reload) only received *future* changes, because the
  running monitor emits on change. `start_watching` now returns the current
  value, which the `watch*` call delivers to its own callback only; monitors
  emit only on change, so existing subscribers no longer see duplicates and
  the first subscriber gets the initial value exactly once.
- **iOS / Android: unknown battery level reported as 0%.** iOS sent `0` when
  `batteryLevel` is unknown (always on the Simulator) and Android passed through
  an unsupported `BATTERY_PROPERTY_CAPACITY`; `level` is now omitted instead
  (Android falls back to the battery intent's level/scale first).
- **iOS release builds failed to link** with `Undefined symbols:
  _init_plugin_device_info` on Xcode 27 / Swift 6.4, which hides `internal`
  symbols when merging a package's objects in release. The Swift entry point is
  now `public`.

## [1.1.1] - 2026-11-17

### Fixed

- **Android: Kotlin 2.x compatibility** — replaced the deprecated
  `kotlinOptions { jvmTarget = "1.8" }` block in `android/build.gradle.kts`
  with `kotlin { compilerOptions { jvmTarget = JvmTarget.JVM_1_8 } }`. Since
  Kotlin 2.2 the `kotlinOptions` DSL is an error, which broke Android builds of
  consuming apps on Kotlin Gradle plugin 2.x. The new form matches Tauri's own
  Android module and still works with Kotlin 1.9, so consumers are not forced to
  upgrade. No Rust or JS API changes; returned device information is unchanged.

## [1.1.0] - 2026-08-24

### Added

- **Reactive Watch API** — subscribe to a kind of device information and receive
  a callback whenever the value changes, instead of polling a getter yourself.
  The current value is delivered immediately, then on every change.
  - `watchBattery`, `watchNetwork`, `watchStorage`, `watchDisplay`, and
    `watchDevice`, each returning a `Promise<UnwatchFn>`.
  - `WatchOptions` (`intervalMs`) and `UnwatchFn` types, exported for consumers.
  - `start_watching` / `stop_watching` commands, with matching
    `allow-start-watching` and `allow-stop-watching` permissions (included in
    `device-info:default`).
- **Native, event-driven monitors on macOS** — battery via IOKit power
  notifications, display via Core Graphics reconfiguration callbacks, and
  network via SystemConfiguration reachability. The CPU stays idle between
  changes and updates arrive the instant the OS reports them.
- **Change-detecting polling fallback** for kinds/platforms without a native
  event source (`storage` and `device` everywhere; all kinds off macOS). Events
  are emitted only when the value actually changes.
- Reference-counted monitors: subscribers to the same kind share a single
  monitor that starts on the first subscriber and is torn down on the last.
- Documentation for the watch API (API reference page, guide section, examples)
  and unit tests covering the monitor lifecycle for every kind.

### Performance

- Per-kind polling defaults and floors so expensive getters are not polled
  aggressively: `device` defaults to 60000 ms (min 10000 ms), `storage` to
  10000 ms (min 1000 ms), others to 2000 ms (min 250 ms). Notably, `device`
  (which shells out to `system_profiler` on macOS and rarely changes) is no
  longer read every couple of seconds.
- The poller now sleeps for the full interval on a condition variable and wakes
  immediately on stop, instead of busy-waking every 100 ms — letting the CPU
  idle and cooperating with power-saving.
- Change events are emitted by reference, avoiding a deep clone of each snapshot.

### Fixed

- Removed a run-loop teardown race on macOS that could hang `stop_watching`
  indefinitely when a watcher was stopped just after starting; stop is now
  signalled through the run loop itself and is race-free.
- `stop_watching` no longer holds the shared watcher lock while tearing down a
  monitor, so a slow teardown can't block subscribe/unsubscribe for other kinds.
- Contained panics inside the macOS OS callbacks so they can no longer unwind
  across the C ABI boundary and abort the process.
- The example app surfaces watcher start failures instead of leaving an
  unhandled promise rejection.

## [1.0.1]

- Documentation updates, crates.io metadata, and a Tests CI badge.

## [1.0.0]

- Initial release: `getDeviceInfo`, `getBatteryInfo`, `getNetworkInfo`,
  `getStorageInfo`, and `getDisplayInfo` across Windows, macOS, Linux, iOS, and
  Android.

[1.3.1]: https://github.com/edisdev/tauri-plugin-device-info/releases/tag/v1.3.1
[1.3.0]: https://github.com/edisdev/tauri-plugin-device-info/releases/tag/v1.3.0
[1.2.0]: https://github.com/edisdev/tauri-plugin-device-info/releases/tag/v1.2.0
[1.1.1]: https://github.com/edisdev/tauri-plugin-device-info/releases/tag/v1.1.1
[1.1.0]: https://github.com/edisdev/tauri-plugin-device-info/releases/tag/v1.1.0
[1.0.1]: https://github.com/edisdev/tauri-plugin-device-info/releases/tag/v1.0.1
[1.0.0]: https://github.com/edisdev/tauri-plugin-device-info/releases/tag/v1.0.0
