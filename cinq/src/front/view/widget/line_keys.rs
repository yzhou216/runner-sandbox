// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Region that gives the keys of Vim's command line that recall a search to
//! the search line
//!
//! iced's text field recalls nothing: it lets the arrows pass.  The region
//! takes them before the field sees them and sends them as messages, since
//! the app holds the history, and the field shows what the app says.

use iced::{
    Element, Event,
    advanced::{Shell, layout::Layout, mouse},
};

use super::{
    key_pressed,
    wrap::{Intercept, wrap},
};
use crate::front::{Message, keymap::KeyPress, message::LineKey};

pub fn line_keys<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    wrap(content, Line)
}

struct Line;

impl Intercept for Line {
    fn before(
        &self,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let line = key_pressed(event)
            .and_then(|KeyPress { press, .. }| press)
            .and_then(LineKey::of);
        if let Some(line) = line {
            shell.publish(Message::SearchKey(line));
            shell.capture_event();
        }
        line.is_none()
    }
}
