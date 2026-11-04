// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Entries of the I/O Registry.

use std::{ffi::CStr, ptr};

use crate::{
    macos::{cf::Cf, sys},
    reverser::Device,
};

/// Driver class of multitouch trackpads, some of whose services lack a
/// scroll acceleration type
const TRACKPAD_DRIVER: &CStr = c"AppleMultitouchTrackpadHIDEventDriver";

/// Owned handle to an I/O Registry entry
pub struct Service(sys::io_service_t);

impl Service {
    pub fn with_id(id: u64) -> Option<Self> {
        // SAFETY: `IOServiceGetMatchingService` consumes the matching
        // dictionary
        let service = unsafe {
            sys::IOServiceGetMatchingService(
                sys::kIOMainPortDefault,
                sys::IORegistryEntryIDMatching(id).cast_const(),
            )
        };
        (service != sys::IO_OBJECT_NULL).then_some(Self(service))
    }

    /// Classifies the entry the way Darwin picks between the mouse and
    /// trackpad scroll settings.
    pub fn device(&self) -> Device {
        let trackpad = self.string(c"HIDScrollAccelerationType").as_deref()
            == Some("HIDTrackpadScrollAcceleration")
            || self.conforms_to(TRACKPAD_DRIVER)
            || self
                .parent()
                .is_some_and(|parent| parent.conforms_to(TRACKPAD_DRIVER));
        if trackpad {
            Device::Trackpad
        } else {
            Device::Mouse
        }
    }

    pub fn product(&self) -> Option<String> {
        self.string(c"Product")
    }

    fn parent(&self) -> Option<Self> {
        let mut parent = sys::IO_OBJECT_NULL;
        // SAFETY: The plane name is NUL-terminated, and `parent` is writable
        let status = unsafe {
            sys::IORegistryEntryGetParentEntry(
                self.0,
                sys::kIOServicePlane.as_ptr(),
                &raw mut parent,
            )
        };
        (status == sys::KERN_SUCCESS).then_some(Self(parent))
    }

    fn conforms_to(&self, class: &CStr) -> bool {
        // SAFETY: `class` is NUL-terminated
        unsafe { sys::IOObjectConformsTo(self.0, class.as_ptr()) != 0 }
    }

    fn string(&self, key: &CStr) -> Option<String> {
        let key = Cf::string(key)?;
        // SAFETY: `key` is a string, and the property is a new reference
        unsafe {
            Cf::from_owned(sys::IORegistryEntryCreateCFProperty(
                self.0,
                key.as_ptr().cast(),
                ptr::null(),
                0,
            ))
        }
        .and_then(|value| value.text())
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        // SAFETY: `self` owns the handle
        unsafe { sys::IOObjectRelease(self.0) };
    }
}
