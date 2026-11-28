// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Widgets that iced lacks
//!
//! Most of them work around an iced widget with [`wrap`].  The checkbox and
//! the drop-down hold the keyboard focus themselves, and the pictures draw
//! through a shader of their own.

mod button;
mod check;
mod dropdown;
pub(super) mod focus;
mod hide;
pub mod hint;
mod keep_keys;
mod line_keys;
mod marked_row;
mod pictures;
mod radio;
mod wrap;

use iced::{Event, Rectangle, mouse, touch};

use crate::front::keymap::KeyPress;
pub use button::{Button, button};
pub use check::{check, check_and_leave};
pub(super) use dropdown::{ELLIPSIS, shortened, shortened_start};
pub use dropdown::{dropdown, width as dropdown_width};
pub use hide::hide;
pub use hint::{Act, Site, Target, hinted, reached, window};
pub use keep_keys::keep_keys;
pub use line_keys::line_keys;
pub use marked_row::marked_row;
pub use pictures::{Layer, pictures};
pub use radio::radio;

fn key_pressed(event: &Event) -> Option<KeyPress<'_>> {
    match event {
        Event::Keyboard(event) => KeyPress::of(event),
        _ => None,
    }
}

fn pressed(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

fn pointer_over(cursor: mouse::Cursor, bounds: Rectangle) -> mouse::Interaction {
    if cursor.is_over(bounds) {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    }
}
