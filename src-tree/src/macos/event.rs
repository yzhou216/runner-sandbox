// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scroll events that pass through the tap.

mod fields;

use std::ptr::NonNull;

use self::fields::Fields;
use super::{hid::HidEvent, sys};
use crate::reverser::Axis;

/// Scroll event lent to the tap callback
pub struct ScrollEvent(NonNull<sys::CGEvent>);

impl ScrollEvent {
    /// # Safety
    ///
    /// `event` must be null or outlive the result.
    pub unsafe fn from_raw(event: sys::CGEventRef) -> Option<Self> {
        NonNull::new(event).map(Self)
    }

    fn integer(&self, field: sys::CGEventField) -> i64 {
        // SAFETY: The event outlives `self`
        unsafe { sys::CGEventGetIntegerValueField(self.0.as_ptr(), field) }
    }

    fn set_integer(&self, field: sys::CGEventField, value: i64) {
        // SAFETY: The event outlives `self`
        unsafe { sys::CGEventSetIntegerValueField(self.0.as_ptr(), field, value) }
    }

    fn double(&self, field: sys::CGEventField) -> f64 {
        // SAFETY: The event outlives `self`
        unsafe { sys::CGEventGetDoubleValueField(self.0.as_ptr(), field) }
    }

    fn set_double(&self, field: sys::CGEventField, value: f64) {
        // SAFETY: The event outlives `self`
        unsafe { sys::CGEventSetDoubleValueField(self.0.as_ptr(), field, value) }
    }

    pub fn phases(&self) -> (i64, i64) {
        (
            self.integer(sys::kCGScrollWheelEventScrollPhase),
            self.integer(sys::kCGScrollWheelEventMomentumPhase),
        )
    }

    /// Returns the distance along `axis` in pixels.
    pub fn points(&self, axis: Axis) -> i64 {
        self.integer(Fields::of(axis).point)
    }

    /// Returns the HID event that the window server embeds in events from
    /// hardware.
    pub fn hid(&self) -> Option<HidEvent> {
        // SAFETY: The event outlives `self`, and the copy is a new reference
        unsafe { HidEvent::from_owned(sys::CGEventCopyIOHIDEvent(self.0.as_ptr())) }
    }

    /// Negates the distance along `axis` in every field that carries it.
    ///
    /// Writing the line delta recomputes the others, so everything is read
    /// first and the line delta written first.  WebKit reads the HID event.
    pub fn reverse(&self, axis: Axis, hid: Option<&HidEvent>) {
        let fields = Fields::of(axis);
        let line = self.integer(fields.line);
        let fixed = self.double(fields.fixed);
        let point = self.integer(fields.point);
        let embedded = hid.map(|hid| (hid, hid.float(fields.hid)));
        self.set_integer(fields.line, line.wrapping_neg());
        self.set_double(fields.fixed, -fixed);
        self.set_integer(fields.point, point.wrapping_neg());
        if let Some((hid, distance)) = embedded {
            hid.set_float(fields.hid, -distance);
        }
    }
}
