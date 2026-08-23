//! Change-detecting poller.
//!
//! Reads the value on an interval and emits an event only when it differs from
//! the last emitted value. Used as the universal fallback for kinds/platforms
//! that have no native event source (storage always; anything not yet wired up
//! natively).
//!
//! The interval defaults are chosen **per kind**: some getters are far more
//! expensive than others (on macOS `device` shells out to `system_profiler`,
//! ~1-2s of CPU) and some values barely ever change. Polling those aggressively
//! would spawn a subprocess every couple of seconds for data that is effectively
//! static, so they default to — and are floored at — much longer intervals.

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Runtime};

use super::{emit_if_changed, event_name, MonitorHandle};

/// Default polling interval for a kind when the caller does not specify one.
///
/// `device` almost never changes and is very expensive to read; `storage`
/// changes slowly. Everything else uses a responsive 2s default.
fn default_interval_ms(kind: &str) -> u64 {
    match kind {
        "device" => 60_000,
        "storage" => 10_000,
        _ => 2_000,
    }
}

/// Lower bound on the polling interval for a kind, so a caller cannot pin an
/// expensive getter to a tiny interval and peg a CPU core.
fn min_interval_ms(kind: &str) -> u64 {
    match kind {
        "device" => 10_000,
        "storage" => 1_000,
        _ => 250,
    }
}

/// Shared stop flag: the boolean guards the condition, the condvar lets `stop`
/// wake the sleeping poller immediately instead of it polling the flag.
type StopSignal = Arc<(Mutex<bool>, Condvar)>;

/// Handle to a running poller thread.
pub(super) struct PollingHandle {
    stop: StopSignal,
}

impl MonitorHandle for PollingHandle {
    fn stop(self: Box<Self>) {
        let (lock, cvar) = &*self.stop;
        *lock.lock().unwrap_or_else(|e| e.into_inner()) = true;
        cvar.notify_all();
    }
}

/// Spawns a poller thread for `kind` and returns its handle.
///
/// The current value is emitted on the first iteration so a fresh subscriber
/// gets the initial state without waiting a full interval.
pub(super) fn spawn<R: Runtime>(
    app: &AppHandle<R>,
    kind: &str,
    interval_ms: Option<u64>,
) -> Box<dyn MonitorHandle> {
    let interval = interval_ms
        .unwrap_or_else(|| default_interval_ms(kind))
        .max(min_interval_ms(kind));
    let stop: StopSignal = Arc::new((Mutex::new(false), Condvar::new()));

    let app = app.clone();
    let kind = kind.to_string();
    let stop_flag = stop.clone();

    std::thread::spawn(move || {
        let event = event_name(&kind);
        let interval = Duration::from_millis(interval);
        let (lock, cvar) = &*stop_flag;
        let mut last: Option<serde_json::Value> = None;

        loop {
            emit_if_changed(&app, &event, &kind, &mut last);

            // Sleep for the full interval, but wake the instant `stop` is called.
            let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            if *guard {
                break;
            }
            let (guard, _) = cvar
                .wait_timeout(guard, interval)
                .unwrap_or_else(|e| e.into_inner());
            let stopped = *guard;
            drop(guard);
            if stopped {
                break;
            }
        }
    });

    Box::new(PollingHandle { stop })
}
