// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! CoreGraphics: event taps and the fields of scroll events.

use std::ffi::c_void;

use super::{CFMachPortRef, IOHIDEventRef};

opaque!(CGEvent);

pub type CGEventRef = *mut CGEvent;
pub type CGEventTapProxy = *mut c_void;
pub type CGEventType = u32;
pub type CGEventField = u32;
pub type CGEventMask = u64;
pub type CGEventTapLocation = u32;
pub type CGEventTapPlacement = u32;
pub type CGEventTapOptions = u32;
pub type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    kind: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

pub const kCGSessionEventTap: CGEventTapLocation = 1;
pub const kCGTailAppendEventTap: CGEventTapPlacement = 1;
pub const kCGEventTapOptionDefault: CGEventTapOptions = 0;

pub const kCGEventScrollWheel: CGEventType = 22;
pub const kCGEventTapDisabledByTimeout: CGEventType = 0xFFFF_FFFE;
pub const kCGEventTapDisabledByUserInput: CGEventType = 0xFFFF_FFFF;

pub const kCGScrollWheelEventDeltaAxis1: CGEventField = 11;
pub const kCGScrollWheelEventDeltaAxis2: CGEventField = 12;
pub const kCGScrollWheelEventFixedPtDeltaAxis1: CGEventField = 93;
pub const kCGScrollWheelEventFixedPtDeltaAxis2: CGEventField = 94;
pub const kCGScrollWheelEventPointDeltaAxis1: CGEventField = 96;
pub const kCGScrollWheelEventPointDeltaAxis2: CGEventField = 97;
pub const kCGScrollWheelEventScrollPhase: CGEventField = 99;
pub const kCGScrollWheelEventMomentumPhase: CGEventField = 123;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: CGEventMask,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

    pub fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> i64;
    pub fn CGEventSetIntegerValueField(event: CGEventRef, field: CGEventField, value: i64);
    pub fn CGEventGetDoubleValueField(event: CGEventRef, field: CGEventField) -> f64;
    pub fn CGEventSetDoubleValueField(event: CGEventRef, field: CGEventField, value: f64);

    /// Returns a new reference to the HID event embedded in `event`, or null
    pub fn CGEventCopyIOHIDEvent(event: CGEventRef) -> IOHIDEventRef;
}
