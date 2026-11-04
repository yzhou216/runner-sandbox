// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! HID events that the window server embeds in scroll events from hardware.

use std::ptr::NonNull;

use super::sys;

/// HID event embedded in a scroll event, released on drop
pub struct HidEvent(NonNull<sys::IOHIDEvent>);

impl HidEvent {
    /// # Safety
    ///
    /// `event` must be null or an owned HID event reference.
    pub unsafe fn from_owned(event: sys::IOHIDEventRef) -> Option<Self> {
        NonNull::new(event).map(Self)
    }

    /// Returns the registry entry ID of the service that sent the event, or
    /// `None` when the sender is unknown.
    pub fn sender(&self) -> Option<u64> {
        // SAFETY: `self` holds a reference to the event
        let sender = unsafe { sys::IOHIDEventGetSenderID(self.0.as_ptr()) };
        (sender != 0).then_some(sender)
    }

    pub fn float(&self, field: sys::IOHIDEventField) -> f64 {
        // SAFETY: `self` holds a reference to the event
        unsafe { sys::IOHIDEventGetFloatValue(self.0.as_ptr(), field) }
    }

    pub fn set_float(&self, field: sys::IOHIDEventField, value: f64) {
        // SAFETY: `self` holds a reference to the event
        unsafe { sys::IOHIDEventSetFloatValue(self.0.as_ptr(), field, value) }
    }
}

impl Drop for HidEvent {
    fn drop(&mut self) {
        // SAFETY: `self` owns one reference
        unsafe { sys::CFRelease(self.0.as_ptr().cast_const().cast()) }
    }
}
