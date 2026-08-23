//! Reactive watch API for the device-info plugin.
//!
//! Consumers *subscribe* to a kind of device information and receive a Tauri
//! event whenever the value changes, instead of polling a getter on a timer.
//!
//! ## Architecture
//!
//! Each watch `kind` is backed by a [`MonitorHandle`]:
//!
//! - When the platform exposes a **native, event-driven** source for a kind
//!   (e.g. IOKit power notifications on macOS), that is used — the CPU stays
//!   idle between changes and updates are delivered the instant they happen.
//! - Otherwise we fall back to a **change-detecting poller** (see [`polling`]).
//!
//! Either way the public API and the emitted event names are identical, so the
//! engine can be upgraded per platform/kind without affecting consumers.
//!
//! Subscribers are reference-counted: the monitor for a kind starts on the
//! first subscriber and is torn down once the last one unsubscribes.

use std::collections::HashMap;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::DeviceInfoExt;

mod native;
mod polling;

/// The kinds of device information that can be watched.
pub(crate) const WATCH_KINDS: &[&str] = &["battery", "network", "storage", "display", "device"];

/// A running change-event source for one kind.
///
/// `stop` must tear down whatever the monitor set up — OS callbacks, run loops,
/// D-Bus connections, or polling threads.
pub(crate) trait MonitorHandle: Send {
    fn stop(self: Box<Self>);
}

/// One active monitor plus its subscriber count.
struct Subscription {
    handle: Box<dyn MonitorHandle>,
    refs: usize,
}

/// Shared state holding at most one monitor per active watch kind.
#[derive(Default)]
pub(crate) struct WatcherState {
    subs: Mutex<HashMap<String, Subscription>>,
}

/// The event name emitted for a given watch kind, e.g. `device-info://battery-changed`.
pub(crate) fn event_name(kind: &str) -> String {
    format!("device-info://{kind}-changed")
}

/// Reads the current value for `kind` as a JSON value, for change comparison and emission.
///
/// Shared by the poller and by native monitors (which read the fresh value when
/// the OS signals a change).
pub(crate) fn read_snapshot<R: Runtime>(
    app: &AppHandle<R>,
    kind: &str,
) -> crate::Result<serde_json::Value> {
    let di = app.device_info();
    let value = match kind {
        "battery" => serde_json::to_value(di.get_battery_info()?),
        "network" => serde_json::to_value(di.get_network_info()?),
        "storage" => serde_json::to_value(di.get_storage_info()?),
        "display" => serde_json::to_value(di.get_display_info()?),
        "device" => serde_json::to_value(di.get_device_info()?),
        other => {
            return Err(crate::Error::DeviceInfo(format!(
                "unknown watch kind: {other}"
            )))
        }
    };
    value.map_err(|e| crate::Error::DeviceInfo(e.to_string()))
}

/// Reads the current value for `kind` and emits `event` only if it differs from
/// `last`, updating `last` on emit.
///
/// This is the single change-detection + emission path shared by the poller and
/// the native monitors, so both stay in lockstep. The payload is emitted by
/// reference to avoid a deep clone of the value also stored in `last`.
pub(crate) fn emit_if_changed<R: Runtime>(
    app: &AppHandle<R>,
    event: &str,
    kind: &str,
    last: &mut Option<serde_json::Value>,
) {
    if let Ok(snapshot) = read_snapshot(app, kind) {
        if last.as_ref() != Some(&snapshot) {
            let _ = app.emit(event, &snapshot);
            *last = Some(snapshot);
        }
    }
}

/// Subscribes to a watch `kind`, starting a monitor if one isn't already running.
///
/// A native event-driven monitor is preferred; if the platform has none for this
/// kind, a change-detecting poller is used (`interval_ms` applies only then, and
/// only for the first subscriber).
pub(crate) fn start<R: Runtime>(
    app: &AppHandle<R>,
    kind: &str,
    interval_ms: Option<u64>,
) -> crate::Result<()> {
    if !WATCH_KINDS.contains(&kind) {
        return Err(crate::Error::DeviceInfo(format!(
            "unknown watch kind: {kind}"
        )));
    }

    let state = app.state::<WatcherState>();
    let mut subs = state.subs.lock().map_err(poisoned)?;

    // A monitor is already running for this kind: just add a subscriber.
    if let Some(sub) = subs.get_mut(kind) {
        sub.refs += 1;
        return Ok(());
    }

    // Prefer a native event-driven monitor; fall back to the poller.
    let handle = match native::try_spawn(app, kind)? {
        Some(handle) => handle,
        None => polling::spawn(app, kind, interval_ms),
    };

    subs.insert(kind.to_string(), Subscription { handle, refs: 1 });
    Ok(())
}

/// Unsubscribes from a watch `kind`, stopping the monitor once the last subscriber leaves.
pub(crate) fn stop<R: Runtime>(app: &AppHandle<R>, kind: &str) -> crate::Result<()> {
    let state = app.state::<WatcherState>();

    // Take the handle out from under the lock, then tear it down *after* releasing
    // it: `handle.stop()` can block (it joins the monitor thread), and holding the
    // subs mutex across that would serialize every other kind's subscribe/unsubscribe.
    let handle = {
        let mut subs = state.subs.lock().map_err(poisoned)?;
        match subs.get_mut(kind) {
            Some(sub) => {
                sub.refs = sub.refs.saturating_sub(1);
                if sub.refs == 0 {
                    subs.remove(kind).map(|sub| sub.handle)
                } else {
                    None
                }
            }
            None => None,
        }
    };

    if let Some(handle) = handle {
        handle.stop();
    }
    Ok(())
}

/// Maps a poisoned-lock error into the plugin's error type.
fn poisoned<E: std::fmt::Display>(e: E) -> crate::Error {
    crate::Error::DeviceInfo(format!("watcher state poisoned: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_name_follows_convention() {
        assert_eq!(event_name("battery"), "device-info://battery-changed");
        assert_eq!(event_name("network"), "device-info://network-changed");
    }

    #[test]
    fn watch_kinds_cover_all_getters() {
        for kind in ["battery", "network", "storage", "display", "device"] {
            assert!(WATCH_KINDS.contains(&kind), "missing watch kind: {kind}");
        }
    }

    // The tests below use Tauri's mock runtime, which is only available off
    // Windows here (see the target-gated dev-dependency in Cargo.toml).

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn unknown_kind_is_rejected() {
        let app = mock_app();
        let err = start(app.handle(), "gpu", None).unwrap_err();
        assert!(err.to_string().contains("unknown watch kind"));
    }

    /// Exercises the full monitor lifecycle (spawn → emit → stop → join → free)
    /// for every kind, including the native macOS event-driven paths. Catches
    /// FFI signature mistakes, use-after-free, and stop deadlocks at runtime.
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn start_and_stop_every_kind_does_not_crash() {
        let app = mock_app();
        for kind in WATCH_KINDS {
            start(app.handle(), kind, Some(250)).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(60));
            stop(app.handle(), kind).unwrap();
        }
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn reference_counting_keeps_monitor_until_last_unsubscribe() {
        let app = mock_app();
        start(app.handle(), "battery", None).unwrap();
        start(app.handle(), "battery", None).unwrap();
        stop(app.handle(), "battery").unwrap(); // one subscriber left
        stop(app.handle(), "battery").unwrap(); // last subscriber → tears down
                                                // Extra stop on an already-removed kind must be a no-op, not a panic.
        stop(app.handle(), "battery").unwrap();
    }

    #[cfg(not(target_os = "windows"))]
    fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .plugin(crate::init())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("failed to build mock app")
    }
}
