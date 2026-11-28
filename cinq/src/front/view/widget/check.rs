// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Checkbox that Tab reaches and Enter flips
//!
//! iced's checkbox cannot take the keyboard focus, so this one holds it in a
//! [`Focused`] widget around iced's checkbox.  A checkbox that cannot change
//! takes no keyboard.

use iced::Element;

use super::{focus::Focused, hinted};
use crate::front::{Message, keymap::Keymap};

/// Returns a checkbox that sends what `on_toggle` makes of the state that a
/// click or Enter flips it to, or one that cannot change without it
pub fn check<'a>(
    label: &'a str,
    checked: bool,
    on_toggle: Option<impl Fn(bool) -> Message + Clone + 'a>,
    keymap: &'a Keymap,
) -> Element<'a, Message> {
    checkbox(label, checked, on_toggle, false, keymap)
}

/// Returns a checkbox as [`check`] does, one that gives the keyboard back to
/// the cue list after Enter flips it, for a setting that the operator makes
/// before moving on to the cues
pub fn check_and_leave<'a>(
    label: &'a str,
    checked: bool,
    on_toggle: Option<impl Fn(bool) -> Message + Clone + 'a>,
    keymap: &'a Keymap,
) -> Element<'a, Message> {
    checkbox(label, checked, on_toggle, true, keymap)
}

fn checkbox<'a>(
    label: &'a str,
    checked: bool,
    on_toggle: Option<impl Fn(bool) -> Message + Clone + 'a>,
    leaves: bool,
    keymap: &'a Keymap,
) -> Element<'a, Message> {
    let flip = on_toggle.clone().map(|on_toggle| on_toggle(!checked));
    let look = iced::widget::checkbox(checked)
        .label(label)
        .on_toggle_maybe(on_toggle)
        .into();
    let check = Element::new(Focused::check(look, flip.clone(), leaves, keymap));
    match flip {
        Some(flip) => hinted(check, flip),
        None => check,
    }
}
