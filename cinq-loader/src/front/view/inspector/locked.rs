// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Value that the operator cannot change now, in the look of a disabled field
//!
//! A disabled text field still takes the keyboard, where nothing shows it,
//! so that Tab stops there for no reason.  This box draws as the field does
//! but takes no part in the order of the keyboard.

use iced::{
    Element, Length, Theme,
    widget::{
        container, row, text,
        text::Wrapping,
        text_input::{self, Status},
    },
};

use crate::front::{
    Message,
    view::{SPACING, line_height},
};

/// Returns `value` in a box as wide as `width` and as high as a field, so
/// that the inspector keeps its height as the selection moves, with the dim
/// `placeholder` where the value is empty, and then `reason` in dim text
/// where something tells why the value is ignored
pub(super) fn locked<'a>(
    value: &str,
    placeholder: &str,
    reason: Option<&'static str>,
    width: Length,
) -> Element<'a, Message> {
    let value = match (value.is_empty(), placeholder.is_empty()) {
        (false, _) => Some(text(value.to_owned())),
        (true, false) => Some(text(placeholder.to_owned()).style(text::secondary)),
        (true, true) => None,
    }
    .map(|value| value.wrapping(Wrapping::None));
    let reason = reason.map(|reason| {
        text(format!("({reason})"))
            .style(text::secondary)
            .wrapping(Wrapping::None)
    });
    container(row![].push(value).push(reason).spacing(SPACING))
        .padding(text_input::DEFAULT_PADDING)
        .height(height())
        .width(width)
        .clip(true)
        .style(disabled)
        .into()
}

fn height() -> f32 {
    line_height() + text_input::DEFAULT_PADDING.y()
}

/// Draws on the window background, since the dim text reads at 7:1 there and
/// not on the dim fill of a disabled field
fn disabled(theme: &Theme) -> container::Style {
    let style = text_input::default(theme, Status::Disabled);
    container::Style {
        text_color: Some(style.value),
        background: Some(theme.extended_palette().background.base.color.into()),
        border: style.border,
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::{Pixels, widget::text::LineHeight};

    use super::*;

    #[test]
    fn a_locked_box_is_as_high_as_the_text_field_that_it_stands_for() {
        let field = f32::from(LineHeight::default().to_absolute(Pixels(16.0)))
            + text_input::DEFAULT_PADDING.top
            + text_input::DEFAULT_PADDING.bottom;
        assert!((height() - field).abs() < f32::EPSILON);
    }
}
