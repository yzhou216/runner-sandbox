// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Event fields that carry the scroll distance along each axis.

use crate::{macos::sys, reverser::Axis};

/// Fields that hold the scroll distance along one axis
pub struct Fields {
    /// Whole lines
    pub line: sys::CGEventField,
    /// Lines in 16.16 fixed point
    pub fixed: sys::CGEventField,
    /// Pixels
    pub point: sys::CGEventField,
    /// Distance in the embedded HID event
    pub hid: sys::IOHIDEventField,
}

impl Fields {
    pub const fn of(axis: Axis) -> Self {
        match axis {
            Axis::Vertical => Self {
                line: sys::kCGScrollWheelEventDeltaAxis1,
                fixed: sys::kCGScrollWheelEventFixedPtDeltaAxis1,
                point: sys::kCGScrollWheelEventPointDeltaAxis1,
                hid: sys::kIOHIDEventFieldScrollY,
            },
            Axis::Horizontal => Self {
                line: sys::kCGScrollWheelEventDeltaAxis2,
                fixed: sys::kCGScrollWheelEventFixedPtDeltaAxis2,
                point: sys::kCGScrollWheelEventPointDeltaAxis2,
                hid: sys::kIOHIDEventFieldScrollX,
            },
        }
    }
}
