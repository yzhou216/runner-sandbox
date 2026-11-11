// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Labels of the list and its menu, each on one line in the default font
//! and size, drawn from its top left corner

use iced::{
    Point, Rectangle, Renderer, Size,
    advanced::text::{self, Renderer as _, Text},
    alignment,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::PADDING;
use crate::front::view::char_width;

pub(super) fn line_height(renderer: &Renderer) -> f32 {
    f32::from(text::LineHeight::default().to_absolute(renderer.default_size()))
}

pub(super) fn label<C>(renderer: &Renderer, content: C, width: f32) -> Text<C> {
    Text {
        content,
        bounds: Size::new(width, line_height(renderer)),
        size: renderer.default_size(),
        line_height: text::LineHeight::default(),
        font: renderer.default_font(),
        align_x: text::Alignment::Default,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::default(),
        wrapping: text::Wrapping::None,
    }
}

/// Returns `name` cut short to fit in `room`, with an ellipsis in place of
/// the rest.  The font is monospace, so that the room holds a number of
/// columns.
pub(super) fn fitted(name: &str, room: f32) -> String {
    let columns = (room / char_width()).floor() as usize;
    if name.width() <= columns {
        return name.to_owned();
    }
    let kept: String = name
        .chars()
        .scan(0, |taken, character| {
            *taken += character.width().unwrap_or(0);
            (*taken < columns).then_some(character)
        })
        .collect();
    format!("{kept}\u{2026}")
}

/// Labels go by their top left corner rather than by their middle, as
/// iced's own pick list draws them, since iced's software renderer redraws
/// a changed text only below and to the right of the point it is drawn at.
pub(super) fn start(renderer: &Renderer, bounds: Rectangle) -> Point {
    Point::new(
        bounds.x + PADDING.left,
        bounds.center_y() - line_height(renderer) / 2.0,
    )
}
