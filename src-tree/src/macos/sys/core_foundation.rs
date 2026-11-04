// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! CoreFoundation: object lifetimes, strings, dictionaries, and run loops.

use std::ffi::{c_char, c_void};

opaque!(
    CFBoolean,
    CFDictionary,
    CFDictionaryKeyCallBacks,
    CFDictionaryValueCallBacks,
    CFMachPort,
    CFRunLoop,
    CFRunLoopSource,
    CFString,
);

pub type Boolean = u8;
pub type CFIndex = isize;
pub type CFTypeID = usize;
pub type CFTypeRef = *const c_void;
pub type CFAllocatorRef = *const c_void;
pub type CFBooleanRef = *const CFBoolean;
pub type CFDictionaryRef = *const CFDictionary;
pub type CFMutableDictionaryRef = *mut CFDictionary;
pub type CFMachPortRef = *mut CFMachPort;
pub type CFRunLoopRef = *mut CFRunLoop;
pub type CFRunLoopSourceRef = *mut CFRunLoopSource;
pub type CFStringRef = *const CFString;
pub type CFStringEncoding = u32;
pub type CFTimeInterval = f64;
pub type CFRunLoopRunResult = i32;

pub const kCFStringEncodingUTF8: CFStringEncoding = 0x0800_0100;
/// Result of `CFRunLoopRunInMode` when the mode has no sources to wait on
pub const kCFRunLoopRunFinished: CFRunLoopRunResult = 1;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub safe static kCFBooleanTrue: CFBooleanRef;
    pub safe static kCFRunLoopCommonModes: CFStringRef;
    pub safe static kCFRunLoopDefaultMode: CFStringRef;
    pub static kCFTypeDictionaryKeyCallBacks: CFDictionaryKeyCallBacks;
    pub static kCFTypeDictionaryValueCallBacks: CFDictionaryValueCallBacks;

    pub fn CFRelease(cf: CFTypeRef);
    pub fn CFGetTypeID(cf: CFTypeRef) -> CFTypeID;

    pub safe fn CFStringGetTypeID() -> CFTypeID;
    pub fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        c_str: *const c_char,
        encoding: CFStringEncoding,
    ) -> CFStringRef;
    pub fn CFStringGetLength(string: CFStringRef) -> CFIndex;
    pub safe fn CFStringGetMaximumSizeForEncoding(
        length: CFIndex,
        encoding: CFStringEncoding,
    ) -> CFIndex;
    pub fn CFStringGetCString(
        string: CFStringRef,
        buffer: *mut c_char,
        buffer_size: CFIndex,
        encoding: CFStringEncoding,
    ) -> Boolean;

    pub fn CFDictionaryCreate(
        allocator: CFAllocatorRef,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: CFIndex,
        key_call_backs: *const CFDictionaryKeyCallBacks,
        value_call_backs: *const CFDictionaryValueCallBacks,
    ) -> CFDictionaryRef;

    pub fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        port: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    pub safe fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    pub safe fn CFRunLoopRun();
    pub fn CFRunLoopRunInMode(
        mode: CFStringRef,
        seconds: CFTimeInterval,
        return_after_source_handled: Boolean,
    ) -> CFRunLoopRunResult;
}
