// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! ApplicationServices: the Accessibility permission, from HIServices.

use super::{Boolean, CFDictionaryRef, CFStringRef};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    pub safe static kAXTrustedCheckOptionPrompt: CFStringRef;

    pub fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> Boolean;
}
