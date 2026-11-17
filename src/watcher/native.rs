//! Native, OS-event-driven monitors.
//!
//! [`try_spawn`] returns `Ok(Some(handle))` when the current platform has a
//! native event source for `kind`, or `Ok(None)` to fall back to polling.
//!
//! ## Roadmap (each entry requires on-device verification)
//!
//! | Kind    | macOS                                   | Windows                              | Linux                          |
//! |---------|-----------------------------------------|--------------------------------------|--------------------------------|
//! | battery | IOKit `IOPSNotificationCreateRunLoopSource` ✅ | `RegisterPowerSettingNotification`   | UPower / D-Bus                 |
//! | display | `CGDisplayRegisterReconfigurationCallback` ✅ | `WM_DISPLAYCHANGE`                   | XRandR                         |
//! | network | `SCNetworkReachability`                 | `NotifyUnicastIpAddressChange`       | netlink / NetworkManager       |
//! | storage | *(no OS event — always polled)*         | *(polled)*                           | *(polled)*                     |
//! | device  | *(rarely changes — polled)*             | *(polled)*                           | *(polled)*                     |

use tauri::{AppHandle, Runtime};

use super::{LastValue, MonitorHandle};

#[cfg(target_os = "macos")]
mod macos;

/// Returns a native event-driven monitor for `kind` if this platform supports
/// one, else `Ok(None)` so the caller falls back to polling.
///
/// The monitor must compare against, and update, the shared `last` (holding its
/// lock across each read-compare-emit), and must only emit on change.
pub(super) fn try_spawn<R: Runtime>(
    app: &AppHandle<R>,
    kind: &str,
    last: LastValue,
) -> crate::Result<Option<Box<dyn MonitorHandle>>> {
    #[cfg(target_os = "macos")]
    {
        macos::try_spawn(app, kind, last)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, kind, last);
        Ok(None)
    }
}
