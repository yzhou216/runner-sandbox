// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The name of the show, which gives way before the hints do
//!
//! A file named with sixty characters would take the whole bar, cut off in
//! the middle of a character, and every hint would give way to it, GO and
//! Panic among them.  The name is cut short at its end with an ellipsis
//! instead, down to [`KEPT`] columns, so that the mode, the warnings, and
//! the keys stay.

use unicode_width::UnicodeWidthStr;

use super::{hints_in, status};
use crate::{
    back::one_line,
    front::{App, view::widget::shortened},
};

/// Fewest columns of the name that stay, its ellipsis among them, before a
/// hint gives way to it
const KEPT: usize = 16;

/// Returns the name of the show as it shows in a bar of `room` columns, and
/// the hints beside it.  The hints fit beside the name at its least, and the
/// name takes the room that they leave.
pub(super) fn shown(app: &App, room: usize) -> (String, String) {
    let name = one_line(&app.show.name());
    let rest = status(app, String::new())
        .iter()
        .map(|part| part.text.width())
        .sum::<usize>();
    let kept = name.width().min(KEPT);
    let hints = hints_in(app, room.saturating_sub(rest + kept));
    let name = shortened(&name, room.saturating_sub(rest + hints.width()));
    (name, hints)
}
