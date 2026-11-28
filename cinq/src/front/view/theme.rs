// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Colors of modus-vivendi, the dark theme of the Modus themes for Emacs
//!
//! Every value below is a named color of modus-vivendi, which keeps the
//! background pure black.  Every color is opaque, and the theme assigns them
//! to the roles that iced's own widget styles draw with, so the widgets keep
//! their default look.  The roles carry the meanings common to interfaces:
//! blue marks what comes next or is chosen, green what runs, yellow what
//! needs a look, and red what failed or stops everything.
//!
//! The exceptions are [`button`], since iced draws disabled buttons half
//! transparent, so buttons get a style of their own, which [`go`] and
//! [`panic`](fn@panic) paint green and red, [`field`], since iced has no
//! style for a field that holds a mistake and draws the border of a field
//! that has the keyboard no thicker than any other, [`line()`], which drops
//! the border of the search line so that it fits the status bar, and
//! [`LINK`], [`SELECTION`], [`HINT_BACKGROUND`], and [`HINT_TEXT`], since
//! iced has no role for links, for the selected row of a table, or for the
//! labels of hint mode.  On that row the blue, green, and red state words
//! read at 4.5:1 and not at 7:1, since no blue, green, or red of
//! modus-vivendi reaches 7:1 there, and the words carry the meaning, so that
//! the color is never the only sign.

use iced::{
    Border, Color, color,
    theme::{
        Palette, Theme,
        palette::{Background, Danger, Extended, Pair, Primary, Secondary, Success, Warning},
    },
    widget::{
        button::{Status, Style},
        text, text_input,
    },
};

#[cfg(test)]
mod tests;

const BG_MAIN: Color = color!(0x000000);
const BG_DIM: Color = color!(0x1e1e1e);
const BG_INACTIVE: Color = color!(0x303030);
const BG_ACTIVE: Color = color!(0x535353);
const BORDER: Color = color!(0x646464);
const BORDER_MODE_LINE_ACTIVE: Color = color!(0x959595);
const FG_DIM: Color = color!(0x989898);
const FG_MAIN: Color = color!(0xffffff);
const BG_HOVER: Color = color!(0x45605e);
const BG_HL_LINE: Color = color!(0x2f3849);
const BG_COMPLETION: Color = color!(0x2f447f);
const BLUE: Color = color!(0x2fafff);
/// Color of the keys that a waiting key sequence offers, since it reads at 7:1
/// on the dim background, which blue does not
pub(super) const BLUE_COOLER: Color = color!(0x00bcff);
const BLUE_WARMER: Color = color!(0x79a8ff);
const GREEN: Color = color!(0x44bc44);
const YELLOW_WARMER: Color = color!(0xfec43f);
const RED: Color = color!(0xff5f59);
const RED_COOLER: Color = color!(0xff7f9f);
const BG_GREEN_SUBTLE: Color = color!(0x00422a);
const BG_GREEN_INTENSE: Color = color!(0x2f822f);
const BG_YELLOW_SUBTLE: Color = color!(0x4a4000);
const BG_YELLOW_INTENSE: Color = color!(0x7a6100);
const BG_RED_SUBTLE: Color = color!(0x620f2a);
const BG_RED_INTENSE: Color = color!(0x9d1f1f);

/// Color of links, which modus-vivendi gives to `fg-link`
pub(super) const LINK: Color = BLUE_WARMER;

/// Background of the selected row of a table, which modus-vivendi gives to
/// `bg-hl-line`, the line of the cursor, and on which every color of text
/// stays legible
pub(super) const SELECTION: Color = BG_HL_LINE;

/// Background of the label that hint mode draws over a place that a click
/// acts on, which modus-vivendi gives to `bg-completion`.  The white of
/// [`HINT_TEXT`] reads on it at 9.3:1.
pub(super) const HINT_BACKGROUND: Color = BG_COMPLETION;

pub(super) const HINT_TEXT: Color = FG_MAIN;

/// Colors of the roles of iced's palette, which spans of rich text take as
/// they are
pub(super) const PALETTE: Palette = Palette {
    background: BG_MAIN,
    text: FG_MAIN,
    primary: BLUE,
    success: GREEN,
    warning: YELLOW_WARMER,
    danger: RED,
};

pub fn modus_vivendi() -> Theme {
    Theme::custom_with_fn("Modus Vivendi", PALETTE, |_| extended())
}

/// Assigns colors to the roles of iced's widget styles.  Drop-down lists,
/// read-only fields, menus, and scroll rails sit on the dim background,
/// while borders, rules, and the empty part of bars and sliders use the
/// border gray.  Blue marks what is filled, chosen, or focused.
/// Placeholders and disabled values take the dim foreground from the
/// secondary color.
fn extended() -> Extended {
    let pair = |color, text| Pair { color, text };
    Extended {
        background: Background {
            base: pair(BG_MAIN, FG_MAIN),
            weakest: pair(BG_DIM, FG_MAIN),
            weaker: pair(BG_DIM, FG_MAIN),
            weak: pair(BG_DIM, FG_MAIN),
            neutral: pair(BG_INACTIVE, FG_MAIN),
            strong: pair(BORDER, FG_MAIN),
            stronger: pair(BORDER_MODE_LINE_ACTIVE, BG_MAIN),
            strongest: pair(FG_DIM, BG_MAIN),
        },
        primary: Primary {
            weak: pair(BG_COMPLETION, FG_MAIN),
            base: pair(BLUE, BG_MAIN),
            strong: pair(BLUE_COOLER, BG_MAIN),
        },
        secondary: Secondary {
            weak: pair(FG_DIM, BG_MAIN),
            base: pair(FG_DIM, BG_MAIN),
            strong: pair(FG_DIM, BG_MAIN),
        },
        success: Success {
            weak: pair(BG_GREEN_SUBTLE, FG_MAIN),
            base: pair(BG_GREEN_INTENSE, FG_MAIN),
            strong: pair(BG_GREEN_INTENSE, FG_MAIN),
        },
        warning: Warning {
            weak: pair(BG_YELLOW_SUBTLE, FG_MAIN),
            base: pair(BG_YELLOW_INTENSE, FG_MAIN),
            strong: pair(BG_YELLOW_INTENSE, FG_MAIN),
        },
        danger: Danger {
            weak: pair(BG_RED_SUBTLE, FG_MAIN),
            base: pair(BG_RED_INTENSE, FG_MAIN),
            strong: pair(BG_RED_INTENSE, FG_MAIN),
        },
        is_dark: true,
    }
}

pub(super) fn button(theme: &Theme, status: Status) -> Style {
    painted(theme, status, BG_ACTIVE)
}

pub(super) fn go(theme: &Theme, status: Status) -> Style {
    painted(theme, status, BG_GREEN_INTENSE)
}

/// GO in the style of a disabled button, whatever it is doing, for while it
/// would fire nothing, so that its green never reads as ready
pub(super) fn go_without_cues(theme: &Theme, _: Status) -> Style {
    painted(theme, Status::Disabled, BG_GREEN_INTENSE)
}

pub(super) fn panic(theme: &Theme, status: Status) -> Style {
    painted(theme, status, BG_RED_INTENSE)
}

/// Thickness of the border of a field or a list that has the keyboard, twice
/// that of one at rest, so that a change of 2.7 to 1 in its color is not the
/// only sign of where the keyboard is
pub(super) const FOCUS_WIDTH: f32 = 2.0;

pub(super) fn field(valid: bool) -> fn(&Theme, text_input::Status) -> text_input::Style {
    if valid { focusable } else { invalid }
}

/// Returns `border` as it draws for a field in `status`, thicker while the
/// field has the keyboard
pub(super) fn focused(border: Border, status: text_input::Status) -> Border {
    match status {
        text_input::Status::Focused { .. } => border.width(FOCUS_WIDTH),
        _ => border,
    }
}

fn focusable(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let style = text_input::default(theme, status);
    text_input::Style {
        border: focused(style.border, status),
        ..style
    }
}

/// Draws a field with no border, so that the search line fits in the single
/// line of the status bar, as Vim's command line does
pub(super) fn line(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let style = text_input::default(theme, status);
    text_input::Style {
        border: style.border.width(0.0),
        background: Color::TRANSPARENT.into(),
        ..style
    }
}

/// Draws the border in red, so that a mistake shows as it is typed, and
/// thicker while the field has the keyboard
fn invalid(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let style = text_input::default(theme, status);
    text_input::Style {
        border: focused(style.border.color(RED), status),
        ..style
    }
}

/// Returns the style of text in the color of what failed, which is red on the
/// black of the window and a lighter red on a row that is selected, where the
/// red of the window has a contrast of 3.9 to 1, below the 4.5 to 1 that text
/// needs
pub(super) fn danger(selected: bool) -> fn(&Theme) -> text::Style {
    if selected {
        |_| text::Style {
            color: Some(RED_COOLER),
        }
    } else {
        text::danger
    }
}

fn painted(theme: &Theme, status: Status, background: Color) -> Style {
    let (background, text) = match status {
        Status::Active | Status::Pressed => (background, FG_MAIN),
        Status::Hovered => (BG_HOVER, FG_MAIN),
        Status::Disabled => (BG_DIM, FG_DIM),
    };
    Style {
        background: Some(background.into()),
        text_color: text,
        ..iced::widget::button::primary(theme, Status::Active)
    }
}
