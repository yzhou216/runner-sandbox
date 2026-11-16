// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the borders that show where the keyboard is, and of the contrast
//! of text on the selected row

use iced::{Color, widget::text_input::Status};

use super::{
    BG_MAIN, BLUE, BLUE_COOLER, FG_MAIN, FOCUS_WIDTH, RED, RED_COOLER, SELECTION, YELLOW_WARMER,
    danger, field, modus_vivendi,
};

#[test]
fn a_focused_field_has_a_border_twice_as_thick_as_one_at_rest() {
    let theme = modus_vivendi();
    let focused = Status::Focused { is_hovered: false };
    let at_rest = field(true)(&theme, Status::Active).border;
    let held = field(true)(&theme, focused).border;
    assert_eq!(at_rest.width, 1.0);
    assert_eq!(held.width, FOCUS_WIDTH);
    assert_eq!(held.width, 2.0 * at_rest.width);
    assert_eq!(held.color, BLUE_COOLER);
    assert_eq!(
        field(true)(&theme, Status::Focused { is_hovered: true })
            .border
            .width,
        FOCUS_WIDTH,
        "the pointer over a focused field does not thin it"
    );
}

#[test]
fn a_field_that_holds_a_rejected_value_keeps_its_red_border_in_both_thicknesses() {
    let theme = modus_vivendi();
    let rejected = field(false);
    let at_rest = rejected(&theme, Status::Active).border;
    let held = rejected(&theme, Status::Focused { is_hovered: false }).border;
    assert_eq!((at_rest.color, at_rest.width), (RED, 1.0));
    assert_eq!((held.color, held.width), (RED, FOCUS_WIDTH));
}

/// Returns the contrast ratio of two colors, as the web content guidelines
/// define it
fn contrast(one: Color, other: Color) -> f32 {
    let light = |channel: f32| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = |c: Color| 0.2126 * light(c.r) + 0.7152 * light(c.g) + 0.0722 * light(c.b);
    let (high, low) = (
        luminance(one).max(luminance(other)),
        luminance(one).min(luminance(other)),
    );
    (high + 0.05) / (low + 0.05)
}

#[test]
fn red_text_needs_the_lighter_red_on_the_selected_row_and_not_elsewhere() {
    assert!(contrast(RED, SELECTION) < 4.5, "why the lighter red exists");
    assert!(contrast(RED_COOLER, SELECTION) >= 4.5);
    assert!(
        contrast(RED, BG_MAIN) >= 4.5,
        "the red of the window serves"
    );
    let style = |selected: bool| danger(selected)(&modus_vivendi()).color;
    assert_eq!(style(true), Some(RED_COOLER));
    assert_eq!(style(false), Some(RED));
}

#[test]
fn the_other_colors_of_text_stay_legible_on_the_selected_row() {
    [FG_MAIN, YELLOW_WARMER, BLUE]
        .into_iter()
        .for_each(|color| {
            assert!(contrast(color, SELECTION) >= 4.5, "{color:?}");
        });
}
