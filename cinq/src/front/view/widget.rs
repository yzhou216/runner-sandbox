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
mod keep_keys;
mod marked_row;
mod pictures;
mod wrap;

use iced::{
    Event, Rectangle,
    keyboard::{self, Key},
    mouse, touch,
};

use crate::front::keymap::Press;
pub use button::button;
pub use dropdown::dropdown;
pub use keep_keys::keep_keys;
pub use marked_row::marked_row;
pub use pictures::{Layer, pictures};

/// Key pressed, with the [`Press`] that the key bindings know it by
struct KeyPress<'a> {
    key: &'a Key,
    press: Option<Press>,
    repeat: bool,
}

fn key_pressed(event: &Event) -> Option<KeyPress<'_>> {
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modified_key,
        modifiers,
        repeat,
        ..
    }) = event
    else {
        return None;
    };
    Some(KeyPress {
        key,
        press: Press::from_key(modified_key, *modifiers),
        repeat: *repeat,
    })
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
