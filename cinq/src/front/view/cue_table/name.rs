// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Name cell of the cue list
//!
//! A name moves right by the depth of its cue and, in a list that holds a
//! group, leaves a gutter of [`INDENT`] columns before it, so that the names
//! of the cues in a group lie one level right of the name of the group.  A
//! group has its mark in the gutter, as a tree view marks its nodes: `-` where
//! the group is expanded and `+` where it is collapsed.  A click on the mark
//! flips the group, and so does its hint label.  A list with no group has no
//! gutter, so that its names lie where they always did.

use iced::{
    Element, Font,
    font::Weight,
    padding,
    widget::{container, mouse_area, row, space, text},
};

use super::{INDENT, ROW_PADDING, cells::broken};
use crate::{
    back::{Cue, Kind},
    front::{
        Message,
        view::{FONT, char_width, hinted},
    },
};

fn mark(cue: &Cue) -> &'static str {
    if cue.collapsed { "+ " } else { "- " }
}

/// Returns the message of a click on the gutter of `cue`, which flips a group
/// and leaves the other cues to the click on their row
fn flip(cue: &Cue) -> Option<Message> {
    (cue.kind == Kind::Group).then_some(Message::FlipGroup(cue.id))
}

/// Returns the cell that shows `name` for `cue`, which lies `depth` levels
/// deep, with `gutter` columns of the font free before the name for the mark
/// of a group
pub(super) fn cell<'a>(cue: &Cue, name: String, depth: u8, gutter: usize) -> Element<'a, Message> {
    let vertical = padding::vertical(ROW_PADDING);
    let indent = space().width(f32::from(depth) * INDENT as f32 * char_width());
    let before: Element<'a, Message> = match flip(cue) {
        Some(flip) => hinted(
            mouse_area(container(text(mark(cue))).padding(vertical)).on_press(flip.clone()),
            flip,
        ),
        None => space().width(gutter as f32 * char_width()).into(),
    };
    let weight = match cue.kind {
        Kind::Group => Weight::Bold,
        _ => Weight::Normal,
    };
    let name = broken(name).font(Font { weight, ..FONT });
    row![indent, before, container(name).padding(vertical)].into()
}

#[cfg(test)]
mod tests;
