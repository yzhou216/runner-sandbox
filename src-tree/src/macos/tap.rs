// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Event tap in the login session and the callback the window server calls.

mod handle;

use std::{cell::OnceCell, ffi::c_void, ptr};

use super::{access, error::Error, event::ScrollEvent, registry::Registry, sys};
use crate::reverser::Reverser;

/// State the tap callback reads, kept alive for the rest of the process
pub struct Tap {
    reverser: Reverser,
    registry: Registry,
    port: OnceCell<sys::CFMachPortRef>,
}

impl Tap {
    /// Creates the tap and adds it to the current run loop.  The state is
    /// leaked, as the callback reads it until the process ends.
    pub fn install(reverser: Reverser) -> Result<(), Error> {
        let tap: &'static Self = Box::leak(Box::new(Self {
            reverser,
            registry: Registry::default(),
            port: OnceCell::new(),
        }));
        let port = access::with_access(|| tap.create())?;
        // Callback needs the port to re-enable the tap
        tap.port.get_or_init(|| port);
        add_to_run_loop(port)
    }

    fn create(&'static self) -> Option<sys::CFMachPortRef> {
        // SAFETY: `callback` has the signature the tap expects, and `self` is
        // leaked, so it outlives the tap
        let port = unsafe {
            sys::CGEventTapCreate(
                sys::kCGSessionEventTap,
                sys::kCGTailAppendEventTap,
                sys::kCGEventTapOptionDefault,
                1 << sys::kCGEventScrollWheel,
                callback,
                ptr::from_ref(self).cast_mut().cast::<c_void>(),
            )
        };
        (!port.is_null()).then_some(port)
    }

    /// Turns the tap back on after the system disabled it, which it does
    /// when a callback takes too long.
    fn enable(&self) {
        if let Some(&port) = self.port.get() {
            // SAFETY: `port` is the live event tap
            unsafe { sys::CGEventTapEnable(port, true) };
            log!("re-enabled the event tap after the system disabled it");
        }
    }
}

fn add_to_run_loop(port: sys::CFMachPortRef) -> Result<(), Error> {
    // SAFETY: `port` is a live event tap
    let source = unsafe { sys::CFMachPortCreateRunLoopSource(ptr::null(), port, 0) };
    (!source.is_null())
        .then(|| {
            // SAFETY: The run loop retains `source`, and `port` is a live tap
            unsafe {
                sys::CFRunLoopAddSource(
                    sys::CFRunLoopGetCurrent(),
                    source,
                    sys::kCFRunLoopCommonModes,
                );
                sys::CGEventTapEnable(port, true);
            }
        })
        .ok_or(Error::Source)
}

/// # Safety
///
/// `info` must be the `Tap` that created the tap.
unsafe extern "C" fn callback(
    _proxy: sys::CGEventTapProxy,
    kind: sys::CGEventType,
    event: sys::CGEventRef,
    info: *mut c_void,
) -> sys::CGEventRef {
    // SAFETY: `Tap::create` registers a leaked `Tap`, which outlives the tap
    let tap = unsafe { &*info.cast_const().cast::<Tap>() };
    match kind {
        sys::kCGEventTapDisabledByTimeout | sys::kCGEventTapDisabledByUserInput => tap.enable(),
        // SAFETY: The window server keeps the event alive until the callback
        // returns
        sys::kCGEventScrollWheel => unsafe { ScrollEvent::from_raw(event) }
            .iter()
            .for_each(|event| tap.handle(event)),
        _ => {}
    }
    event
}
