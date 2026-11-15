// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Content that stays in the widget tree while another page takes its place
//!
//! iced keeps what a widget holds, such as the field that has the keyboard
//! focus and how far a list has scrolled, only while the widget stays in
//! the tree.  Hidden content stays, so that it comes back as it was.  It
//! draws nothing, sees no event, so that no key reaches its fields, and
//! shows none of its menus, which would otherwise draw above the page.

use iced::Element;

use super::wrap::{Intercept, wrap};
use crate::front::Message;

pub fn hide<'a>(content: impl Into<Element<'a, Message>>, hidden: bool) -> Element<'a, Message> {
    wrap(content, Hide(hidden))
}

struct Hide(bool);

impl Intercept for Hide {
    fn shown(&self) -> bool {
        !self.0
    }
}
