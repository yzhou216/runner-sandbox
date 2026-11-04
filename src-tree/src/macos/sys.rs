// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Raw framework bindings, following the Darwin SDK headers.
//!
//! `CGEventCopyIOHIDEvent` and the `IOHIDEvent` functions are private, so
//! their signatures follow Apple's open-source IOHIDFamily.

#![allow(non_camel_case_types, non_upper_case_globals)]

/// Declares types that are only ever handled through pointers
macro_rules! opaque {
    ($($name:ident),* $(,)?) => {$(
        #[repr(C)]
        pub struct $name {
            _data: [u8; 0],
            _marker: std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
        }
    )*};
}

mod application_services;
mod core_foundation;
mod core_graphics;
mod io_kit;

pub use self::{application_services::*, core_foundation::*, core_graphics::*, io_kit::*};
