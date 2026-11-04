// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Owned CoreFoundation objects.

use std::{
    ffi::{CStr, c_char, c_void},
    ptr::{self, NonNull},
};

use super::sys;

/// Owned reference to a CoreFoundation object, released on drop
pub struct Cf(NonNull<c_void>);

impl Cf {
    /// # Safety
    ///
    /// `object` must be null or an owned CoreFoundation reference.
    pub unsafe fn from_owned(object: sys::CFTypeRef) -> Option<Self> {
        NonNull::new(object.cast_mut()).map(Self)
    }

    pub fn string(value: &CStr) -> Option<Self> {
        // SAFETY: `value` is NUL-terminated, and the result is a new reference
        unsafe {
            Self::from_owned(
                sys::CFStringCreateWithCString(
                    ptr::null(),
                    value.as_ptr(),
                    sys::kCFStringEncodingUTF8,
                )
                .cast(),
            )
        }
    }

    /// Creates an immutable dictionary with one entry, retaining both objects.
    pub fn dictionary(key: sys::CFTypeRef, value: sys::CFTypeRef) -> Option<Self> {
        let keys = [key];
        let values = [value];
        // SAFETY: Both arrays hold one object, the callbacks are the standard
        // ones for CoreFoundation objects, and the result is a new reference
        unsafe {
            Self::from_owned(
                sys::CFDictionaryCreate(
                    ptr::null(),
                    keys.as_ptr(),
                    values.as_ptr(),
                    1,
                    &raw const sys::kCFTypeDictionaryKeyCallBacks,
                    &raw const sys::kCFTypeDictionaryValueCallBacks,
                )
                .cast(),
            )
        }
    }

    pub fn as_ptr(&self) -> sys::CFTypeRef {
        self.0.as_ptr().cast_const()
    }

    /// Copies out the contents of a string, or returns `None` for objects of
    /// any other type.
    pub fn text(&self) -> Option<String> {
        Some(self)
            .filter(|object| object.is_string())
            .and_then(Self::string_contents)
    }

    fn is_string(&self) -> bool {
        // SAFETY: `self` holds a live object
        unsafe { sys::CFGetTypeID(self.as_ptr()) == sys::CFStringGetTypeID() }
    }

    fn string_contents(&self) -> Option<String> {
        let string = self.as_ptr().cast::<sys::CFString>();
        // SAFETY: `text` has checked that the object is a string
        let length = unsafe { sys::CFStringGetLength(string) };
        let capacity = sys::CFStringGetMaximumSizeForEncoding(length, sys::kCFStringEncodingUTF8)
            .checked_add(1)?;
        let mut buffer = vec![0_u8; usize::try_from(capacity).ok()?];
        // SAFETY: `buffer` holds `capacity` bytes
        let copied = unsafe {
            sys::CFStringGetCString(
                string,
                buffer.as_mut_ptr().cast::<c_char>(),
                capacity,
                sys::kCFStringEncodingUTF8,
            )
        } != 0;
        copied.then_some(buffer).and_then(|buffer| {
            CStr::from_bytes_until_nul(&buffer)
                .ok()
                .map(|text| text.to_string_lossy().into_owned())
        })
    }
}

impl Drop for Cf {
    fn drop(&mut self) {
        // SAFETY: `self` owns one reference
        unsafe { sys::CFRelease(self.as_ptr()) }
    }
}
