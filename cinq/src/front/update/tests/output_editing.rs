// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that edit cues editing the outputs while they show: the
//! register of outputs, the identity that a cut output takes back, the
//! notices, undo and redo, and the keys that enter the name

mod cutting;
mod notices;
mod registers;

use super::{
    after, locked, on_clipboard,
    output_rows::{ALL, outputs},
    paging::letter,
    press,
};
use crate::front::{App, Message};

/// Types the keys one after the other, each a character without modifiers
pub(super) fn typed(app: App, keys: &str) -> App {
    keys.chars().fold(app, |app, key| {
        app.typed(press(letter(&key.to_string())), false).0
    })
}

pub(super) fn names(app: &App) -> Vec<&str> {
    app.workspace
        .outputs()
        .iter()
        .map(|output| output.name.as_str())
        .collect()
}

/// Returns the app with the outputs showing and the row counted from one
/// selected
pub(super) fn on(row: usize) -> App {
    after(outputs(), [Message::Row(row)])
}

fn without(rows: &[usize]) -> Vec<&'static str> {
    ALL.iter()
        .enumerate()
        .filter(|(row, _)| !rows.contains(row))
        .map(|(_, name)| *name)
        .collect()
}
