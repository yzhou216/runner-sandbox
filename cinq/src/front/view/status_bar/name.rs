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

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{
    fit::{self, ELLIPSIS},
    hints, status, yielding,
};
use crate::front::App;

/// Fewest columns of the name that stay, its ellipsis among them, before a
/// hint gives way to it
const KEPT: usize = 16;

/// Returns `name` cut after as many characters as leave room for an ellipsis
/// in `room` columns, or whole where it fits
pub(in crate::front::view) fn cut(name: &str, room: usize) -> String {
    if name.width() <= room {
        return name.to_owned();
    }
    let text = name
        .chars()
        .scan(0, |columns, char| {
            *columns += char.width().unwrap_or(0);
            Some((*columns, char))
        })
        .take_while(|(columns, _)| columns + ELLIPSIS.width() <= room)
        .map(|(_, char)| char)
        .collect::<String>();
    format!("{text}{ELLIPSIS}")
}

/// Returns the name of the show as it shows in a bar of `room` columns, and
/// the hints beside it.  The hints fit beside the name at its least, and the
/// name takes the room that they leave.
pub(super) fn shown(app: &App, room: usize) -> (String, String) {
    let name = app.show.name();
    let rest = status(app, String::new())
        .iter()
        .map(|part| part.text.width())
        .sum::<usize>();
    let kept = name.width().min(KEPT);
    let hints = fit::shown(
        |told| hints::hints_told(app, told),
        hints::told(app),
        room.saturating_sub(rest + kept),
        yielding(app),
    );
    let name = cut(&name, room.saturating_sub(rest + hints.width()));
    (name, hints)
}
