// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Widgets that iced lacks
//!
//! [`button`](mod@button) acts on the press rather than the release,
//! [`dropdown`](mod@dropdown) lets keys move through its menu,
//! [`keep_keys`](mod@keep_keys) keeps the keys typed into its text fields
//! from reaching the key bindings, [`marked_row`](mod@marked_row) marks the
//! selected row of a table, and [`pictures`](mod@pictures) draws the
//! pictures of video cues, each in the first frame that shows it.  All but
//! the drop-down and the pictures work around an iced widget with [`wrap`],
//! while the drop-down draws itself in the look of iced's pick list, and the
//! pictures draw through a shader of their own.

mod button;
mod dropdown;
mod keep_keys;
mod marked_row;
mod pictures;
mod wrap;

use iced::{Event, Rectangle, mouse, touch};

pub use button::button;
pub use dropdown::dropdown;
pub use keep_keys::keep_keys;
pub use marked_row::marked_row;
pub use pictures::{Layer, pictures};

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
