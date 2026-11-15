// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Widgets that iced lacks
//!
//! All but [`dropdown`](mod@dropdown) and [`pictures`](mod@pictures) work
//! around an iced widget with [`wrap`], while the drop-down draws itself in
//! the look of iced's pick list, and the pictures draw through a shader of
//! their own.

mod button;
mod dropdown;
mod hide;
mod keep_keys;
mod marked_row;
mod pictures;
mod wrap;

use iced::{Event, Rectangle, mouse, touch};

use crate::front::keymap::KeyPress;
pub use button::button;
pub use dropdown::dropdown;
pub use hide::hide;
pub use keep_keys::keep_keys;
pub use marked_row::marked_row;
pub use pictures::{Layer, pictures};

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
