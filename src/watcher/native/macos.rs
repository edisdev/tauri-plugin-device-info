//! macOS native monitors — event-driven via Core Foundation run loops.
//!
//! Each monitor runs a dedicated thread with its own `CFRunLoop` and registers
//! an OS notification source:
//!
//! - **battery** — IOKit `IOPSNotificationCreateRunLoopSource`
//! - **display** — Core Graphics `CGDisplayRegisterReconfigurationCallback`
//! - **network** — SystemConfiguration `SCNetworkReachability`
//!
//! The OS invokes our callback only when state changes, so the CPU stays idle in
//! between. A no-op keep-alive timer is added so `CFRunLoopRun` does not return
//! before `stop` asks it to. Storage and device have no OS change event and fall
//! back to polling.

#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use std::ffi::c_void;
use std::ptr;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use tauri::{AppHandle, Runtime};

use crate::watcher::{emit_if_changed, event_name, MonitorHandle};

// ---- Core Foundation FFI ----------------------------------------------------

type CFRunLoopRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFRunLoopTimerRef = *mut c_void;
type CFStringRef = *const c_void;
type CFAllocatorRef = *const c_void;

type CFRunLoopTimerCallBack = unsafe extern "C" fn(timer: CFRunLoopTimerRef, info: *mut c_void);
type CFRunLoopPerformCallBack = unsafe extern "C" fn(info: *mut c_void);

/// Context for a version-0 `CFRunLoopSource`. Only `perform` is used; the other
/// callbacks are left null (matching how [`SCNetworkReachabilityContext`] is set up).
#[repr(C)]
struct CFRunLoopSourceContext {
    version: isize,
    info: *mut c_void,
    retain: *const c_void,
    release: *const c_void,
    copyDescription: *const c_void,
    equal: *const c_void,
    hash: *const c_void,
    schedule: *const c_void,
    cancel: *const c_void,
    perform: CFRunLoopPerformCallBack,
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFRunLoopDefaultMode: CFStringRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopRun();
    fn CFRunLoopStop(rl: CFRunLoopRef);
    fn CFRunLoopWakeUp(rl: CFRunLoopRef);
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRemoveSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopSourceCreate(
        allocator: CFAllocatorRef,
        order: isize,
        context: *mut CFRunLoopSourceContext,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopSourceSignal(source: CFRunLoopSourceRef);
    fn CFRunLoopAddTimer(rl: CFRunLoopRef, timer: CFRunLoopTimerRef, mode: CFStringRef);
    fn CFRunLoopTimerCreate(
        allocator: CFAllocatorRef,
        fireDate: f64,
        interval: f64,
        flags: usize,
        order: isize,
        callout: CFRunLoopTimerCallBack,
        context: *mut c_void,
    ) -> CFRunLoopTimerRef;
    fn CFAbsoluteTimeGetCurrent() -> f64;
    fn CFRelease(cf: *const c_void);
}

// ---- IOKit (battery) FFI ----------------------------------------------------

type IOPSCallback = unsafe extern "C" fn(context: *mut c_void);

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOPSNotificationCreateRunLoopSource(
        callback: IOPSCallback,
        context: *mut c_void,
    ) -> CFRunLoopSourceRef;
}

// ---- Core Graphics (display) FFI --------------------------------------------

type CGDirectDisplayID = u32;
type CGDisplayChangeSummaryFlags = u32;
type CGDisplayReconfigurationCallBack = unsafe extern "C" fn(
    display: CGDirectDisplayID,
    flags: CGDisplayChangeSummaryFlags,
    userInfo: *mut c_void,
);

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGDisplayRegisterReconfigurationCallback(
        callback: CGDisplayReconfigurationCallBack,
        userInfo: *mut c_void,
    ) -> i32;
    fn CGDisplayRemoveReconfigurationCallback(
        callback: CGDisplayReconfigurationCallBack,
        userInfo: *mut c_void,
    ) -> i32;
}

// ---- SystemConfiguration (network) FFI --------------------------------------

type SCNetworkReachabilityRef = *mut c_void;
type SCNetworkReachabilityFlags = u32;
type SCNetworkReachabilityCallBack = unsafe extern "C" fn(
    target: SCNetworkReachabilityRef,
    flags: SCNetworkReachabilityFlags,
    info: *mut c_void,
);

#[repr(C)]
struct SCNetworkReachabilityContext {
    version: isize,
    info: *mut c_void,
    retain: *const c_void,
    release: *const c_void,
    copyDescription: *const c_void,
}

/// Minimal `sockaddr_in` for a zeroed address ("general internet reachability").
#[repr(C)]
struct SockaddrIn {
    sin_len: u8,
    sin_family: u8,
    sin_port: u16,
    sin_addr: u32,
    sin_zero: [u8; 8],
}

#[link(name = "SystemConfiguration", kind = "framework")]
extern "C" {
    fn SCNetworkReachabilityCreateWithAddress(
        allocator: CFAllocatorRef,
        address: *const c_void,
    ) -> SCNetworkReachabilityRef;
    fn SCNetworkReachabilitySetCallback(
        target: SCNetworkReachabilityRef,
        callout: SCNetworkReachabilityCallBack,
        context: *mut SCNetworkReachabilityContext,
    ) -> u8;
    fn SCNetworkReachabilityScheduleWithRunLoop(
        target: SCNetworkReachabilityRef,
        runLoop: CFRunLoopRef,
        runLoopMode: CFStringRef,
    ) -> u8;
    fn SCNetworkReachabilityUnscheduleFromRunLoop(
        target: SCNetworkReachabilityRef,
        runLoop: CFRunLoopRef,
        runLoopMode: CFStringRef,
    ) -> u8;
}

/// Keep-alive timer firing far in the future, only to keep the run loop alive.
const KEEPALIVE_INTERVAL: f64 = 1.0e10;

// ---- Type-erased emit context -----------------------------------------------

/// Type-erases the generic `AppHandle<R>` so the context can travel through a C `void*`.
trait Fire: Send + Sync {
    fn fire(&self);
}

/// Reads the current value for a kind and emits an event only when it changed.
struct EmitCtx<R: Runtime> {
    app: AppHandle<R>,
    kind: String,
    event: String,
    last: Mutex<Option<serde_json::Value>>,
}

impl<R: Runtime> Fire for EmitCtx<R> {
    fn fire(&self) {
        let Ok(mut last) = self.last.lock() else {
            return;
        };
        emit_if_changed(&self.app, &self.event, &self.kind, &mut last);
    }
}

/// Dereferences a C `void*` back into the boxed [`Fire`] context and fires it.
///
/// # Safety
/// `context` must be the `*mut Box<dyn Fire>` handed to the OS during
/// registration; it stays valid until the monitor thread frees it after the run
/// loop stops.
unsafe fn fire_ctx(context: *mut c_void) {
    let ctx = &*(context as *const Box<dyn Fire>);
    // The platform callbacks below are `extern "C"`; a panic unwinding across that
    // ABI boundary aborts the whole process, so contain it here.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ctx.fire()));
}

unsafe extern "C" fn battery_callback(context: *mut c_void) {
    fire_ctx(context);
}

unsafe extern "C" fn display_callback(
    _display: CGDirectDisplayID,
    _flags: CGDisplayChangeSummaryFlags,
    user_info: *mut c_void,
) {
    fire_ctx(user_info);
}

unsafe extern "C" fn network_callback(
    _target: SCNetworkReachabilityRef,
    _flags: SCNetworkReachabilityFlags,
    info: *mut c_void,
) {
    fire_ctx(info);
}

unsafe extern "C" fn noop_timer(_timer: CFRunLoopTimerRef, _info: *mut c_void) {}

/// Performed on the monitor thread when `stop` signals the stop source. Because
/// it runs *inside* the run loop it can stop it race-free — even a signal that
/// arrives before `CFRunLoopRun` starts is honored the moment the loop runs.
unsafe extern "C" fn stop_perform(_info: *mut c_void) {
    CFRunLoopStop(CFRunLoopGetCurrent());
}

// ---- Run-loop monitor handle ------------------------------------------------

/// Tracks the state of the monitor thread's run loop.
enum LoopState {
    /// The thread has not finished setting up yet.
    Pending,
    /// The run loop is live; holds the `CFRunLoopRef` and the stop-source ref as
    /// `usize`s (so they are `Send`) for `stop` to signal.
    Running { run_loop: usize, stop_source: usize },
}

/// Handle to a macOS run-loop monitor.
struct MacRunLoopHandle {
    state: Arc<(Mutex<LoopState>, Condvar)>,
    join: Option<JoinHandle<()>>,
}

impl MonitorHandle for MacRunLoopHandle {
    fn stop(mut self: Box<Self>) {
        let (lock, cvar) = &*self.state;
        let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        while matches!(*guard, LoopState::Pending) {
            guard = cvar.wait(guard).unwrap_or_else(|e| e.into_inner());
        }
        if let LoopState::Running {
            run_loop,
            stop_source,
        } = *guard
        {
            // SAFETY: signaling the stop source and waking the loop are thread-safe.
            // The source performs on the monitor thread and calls `CFRunLoopStop`
            // there, which is race-free even if the loop has not started running yet
            // (a signaled version-0 source is serviced the moment the loop runs).
            unsafe {
                CFRunLoopSourceSignal(stop_source as CFRunLoopSourceRef);
                CFRunLoopWakeUp(run_loop as CFRunLoopRef);
            }
        }
        drop(guard);

        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

// ---- Platform registration --------------------------------------------------

/// Undoes a registration; called on the monitor thread after the run loop stops.
type Teardown = Box<dyn FnOnce()>;

fn register_battery(run_loop: CFRunLoopRef, ctx: *mut c_void) -> Teardown {
    // SAFETY: valid C callback and context; the source is removed and released in teardown.
    let source = unsafe { IOPSNotificationCreateRunLoopSource(battery_callback, ctx) };
    if source.is_null() {
        return Box::new(|| {});
    }
    // SAFETY: adding our source to this thread's run loop in the default mode.
    unsafe { CFRunLoopAddSource(run_loop, source, kCFRunLoopDefaultMode) };

    let rl = run_loop as usize;
    let src = source as usize;
    Box::new(move || unsafe {
        CFRunLoopRemoveSource(
            rl as CFRunLoopRef,
            src as CFRunLoopSourceRef,
            kCFRunLoopDefaultMode,
        );
        CFRelease(src as *const c_void);
    })
}

fn register_display(_run_loop: CFRunLoopRef, ctx: *mut c_void) -> Teardown {
    // SAFETY: registering a callback with our context; removed in teardown.
    unsafe { CGDisplayRegisterReconfigurationCallback(display_callback, ctx) };
    let ctx_us = ctx as usize;
    Box::new(move || unsafe {
        CGDisplayRemoveReconfigurationCallback(display_callback, ctx_us as *mut c_void);
    })
}

fn register_network(run_loop: CFRunLoopRef, ctx: *mut c_void) -> Teardown {
    // Zeroed IPv4 address = general internet reachability; fires on route changes.
    let addr = SockaddrIn {
        sin_len: core::mem::size_of::<SockaddrIn>() as u8,
        sin_family: 2, // AF_INET
        sin_port: 0,
        sin_addr: 0,
        sin_zero: [0; 8],
    };

    // SAFETY: passing a valid, stack-allocated sockaddr by const pointer.
    let reach = unsafe {
        SCNetworkReachabilityCreateWithAddress(
            ptr::null(),
            &addr as *const SockaddrIn as *const c_void,
        )
    };
    if reach.is_null() {
        eprintln!(
            "device-info: SCNetworkReachabilityCreateWithAddress failed; \
             network watch will report the initial value only"
        );
        return Box::new(|| {});
    }

    let mut context = SCNetworkReachabilityContext {
        version: 0,
        info: ctx,
        retain: ptr::null(),
        release: ptr::null(),
        copyDescription: ptr::null(),
    };

    // SAFETY: `SetCallback` copies the context struct; scheduling on our run loop.
    // Both return a boolean (0 = failure); a failed setup means no change events,
    // so surface it rather than silently degrading to a one-shot value.
    let ok = unsafe {
        let set = SCNetworkReachabilitySetCallback(reach, network_callback, &mut context);
        let sched =
            SCNetworkReachabilityScheduleWithRunLoop(reach, run_loop, kCFRunLoopDefaultMode);
        set != 0 && sched != 0
    };
    if !ok {
        eprintln!(
            "device-info: failed to schedule SCNetworkReachability callback; \
             network watch will report the initial value only"
        );
    }

    let reach_us = reach as usize;
    let rl = run_loop as usize;
    Box::new(move || unsafe {
        SCNetworkReachabilityUnscheduleFromRunLoop(
            reach_us as SCNetworkReachabilityRef,
            rl as CFRunLoopRef,
            kCFRunLoopDefaultMode,
        );
        CFRelease(reach_us as *const c_void);
    })
}

// ---- Public entry point -----------------------------------------------------

/// Returns a native monitor for `kind` if macOS has one, else `Ok(None)` to poll.
pub(super) fn try_spawn<R: Runtime>(
    app: &AppHandle<R>,
    kind: &str,
) -> crate::Result<Option<Box<dyn MonitorHandle>>> {
    let register: fn(CFRunLoopRef, *mut c_void) -> Teardown = match kind {
        "battery" => register_battery,
        "display" => register_display,
        "network" => register_network,
        // storage and device have no OS change event; poll them.
        _ => return Ok(None),
    };
    Ok(Some(spawn_runloop(app, kind, register)))
}

/// Spawns a dedicated run-loop thread, registers the OS source, and returns its handle.
fn spawn_runloop<R: Runtime>(
    app: &AppHandle<R>,
    kind: &str,
    register: fn(CFRunLoopRef, *mut c_void) -> Teardown,
) -> Box<dyn MonitorHandle> {
    let ctx: Box<dyn Fire> = Box::new(EmitCtx {
        app: app.clone(),
        kind: kind.to_string(),
        event: event_name(kind),
        last: Mutex::new(None),
    });
    // A thin pointer to the fat trait-object pointer, so it fits in a C `void*`.
    let ctx_ptr = Box::into_raw(Box::new(ctx)) as usize;

    let state = Arc::new((Mutex::new(LoopState::Pending), Condvar::new()));
    let state_thread = state.clone();

    let join = std::thread::spawn(move || {
        let ctx_void = ctx_ptr as *mut c_void;
        let run_loop = unsafe { CFRunLoopGetCurrent() };

        // Keep-alive timer so `CFRunLoopRun` does not exit when a source is idle.
        // SAFETY: no-op callout, null context; timer is released after the run loop stops.
        let timer = unsafe {
            CFRunLoopTimerCreate(
                ptr::null(),
                CFAbsoluteTimeGetCurrent() + KEEPALIVE_INTERVAL,
                KEEPALIVE_INTERVAL,
                0,
                0,
                noop_timer,
                ptr::null_mut(),
            )
        };
        unsafe { CFRunLoopAddTimer(run_loop, timer, kCFRunLoopDefaultMode) };

        // Stop source: `stop` signals this to make the loop stop *itself* from
        // inside, which is race-free even if `stop` fires before `CFRunLoopRun`.
        // SAFETY: version-0 source with a valid `perform`; removed and released below.
        let mut src_ctx = CFRunLoopSourceContext {
            version: 0,
            info: ptr::null_mut(),
            retain: ptr::null(),
            release: ptr::null(),
            copyDescription: ptr::null(),
            equal: ptr::null(),
            hash: ptr::null(),
            schedule: ptr::null(),
            cancel: ptr::null(),
            perform: stop_perform,
        };
        let stop_source = unsafe { CFRunLoopSourceCreate(ptr::null(), 0, &mut src_ctx) };
        unsafe { CFRunLoopAddSource(run_loop, stop_source, kCFRunLoopDefaultMode) };

        // Platform-specific registration (returns its teardown).
        let teardown = register(run_loop, ctx_void);

        // Publish the run loop + stop source so `stop` can wake it.
        {
            let (lock, cvar) = &*state_thread;
            *lock.lock().unwrap_or_else(|e| e.into_inner()) = LoopState::Running {
                run_loop: run_loop as usize,
                stop_source: stop_source as usize,
            };
            cvar.notify_all();
        }

        // Emit the current value immediately so a fresh subscriber gets initial state.
        // SAFETY: `ctx_ptr` is valid until freed below, after the run loop stops.
        unsafe { fire_ctx(ctx_void) };

        // Blocks until the stop source performs and calls `CFRunLoopStop`.
        unsafe { CFRunLoopRun() };

        // Tear down OS registration, remove/release the stop source and timer,
        // then free the context.
        teardown();
        // SAFETY: the source and timer were created above and are no longer needed
        // once the run loop has stopped.
        unsafe {
            CFRunLoopRemoveSource(run_loop, stop_source, kCFRunLoopDefaultMode);
            CFRelease(stop_source);
            CFRelease(timer);
        };
        // SAFETY: `ctx_ptr` is no longer referenced by any OS callback after teardown.
        unsafe { drop(Box::from_raw(ctx_ptr as *mut Box<dyn Fire>)) };
    });

    Box::new(MacRunLoopHandle {
        state,
        join: Some(join),
    })
}
