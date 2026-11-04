// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! IOKit: the I/O Registry and the HID events embedded in scroll events.

use std::ffi::{CStr, c_char};

use super::{CFAllocatorRef, CFDictionaryRef, CFMutableDictionaryRef, CFStringRef, CFTypeRef};

opaque!(IOHIDEvent);

pub type IOHIDEventRef = *mut IOHIDEvent;
pub type IOHIDEventField = u32;
pub type IOHIDEventSenderID = u64;
pub type IOHIDFloat = f64;

pub type mach_port_t = u32;
pub type io_object_t = mach_port_t;
pub type io_registry_entry_t = io_object_t;
pub type io_service_t = io_object_t;
pub type kern_return_t = i32;
pub type boolean_t = i32;
pub type IOOptionBits = u32;

/// Field base of scroll events, `kIOHIDEventTypeScroll << 16`
const kIOHIDEventFieldScrollBase: IOHIDEventField = 6 << 16;
pub const kIOHIDEventFieldScrollX: IOHIDEventField = kIOHIDEventFieldScrollBase;
pub const kIOHIDEventFieldScrollY: IOHIDEventField = kIOHIDEventFieldScrollBase | 1;

/// Null main port, which IOKit reads as the default one
pub const kIOMainPortDefault: mach_port_t = 0;
pub const kIOServicePlane: &CStr = c"IOService";
pub const IO_OBJECT_NULL: io_object_t = 0;
pub const KERN_SUCCESS: kern_return_t = 0;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    pub safe fn IORegistryEntryIDMatching(entry_id: u64) -> CFMutableDictionaryRef;
    pub fn IOServiceGetMatchingService(
        main_port: mach_port_t,
        matching: CFDictionaryRef,
    ) -> io_service_t;
    pub fn IORegistryEntryCreateCFProperty(
        entry: io_registry_entry_t,
        key: CFStringRef,
        allocator: CFAllocatorRef,
        options: IOOptionBits,
    ) -> CFTypeRef;
    pub fn IORegistryEntryGetParentEntry(
        entry: io_registry_entry_t,
        plane: *const c_char,
        parent: *mut io_registry_entry_t,
    ) -> kern_return_t;
    pub fn IOObjectConformsTo(object: io_object_t, class_name: *const c_char) -> boolean_t;
    pub fn IOObjectRelease(object: io_object_t) -> kern_return_t;

    pub fn IOHIDEventGetSenderID(event: IOHIDEventRef) -> IOHIDEventSenderID;
    pub fn IOHIDEventGetFloatValue(event: IOHIDEventRef, field: IOHIDEventField) -> IOHIDFloat;
    pub fn IOHIDEventSetFloatValue(event: IOHIDEventRef, field: IOHIDEventField, value: IOHIDFloat);
}
