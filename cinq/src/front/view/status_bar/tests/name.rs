// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the name of the show, which gives way to the hints

use unicode_width::UnicodeWidthStr;

use super::{
    super::name::{cut, shown},
    cancel::COLUMNS,
};
use crate::front::{App, update::Show};

/// Returns an app with the show in a file of `name`
fn named(name: &str) -> App {
    App {
        show: Show::in_file(name),
        ..App::default()
    }
}

#[test]
fn a_name_cuts_at_a_character_and_ends_in_an_ellipsis() {
    assert_eq!(cut("show.cinq.toml", 14), "show.cinq.toml");
    assert_eq!(cut("show.cinq.toml", 10), "show.cinq…");
    assert_eq!(cut("日本語の台本.cinq.toml", 9), "日本語の…");
    assert_eq!(cut("show.cinq.toml", 1), "…");
}

#[test]
fn a_short_name_stays_whole_beside_every_hint() {
    let (name, hints) = shown(&named("show.cinq.toml"), COLUMNS);
    assert_eq!(name, "show.cinq.toml");
    assert!(hints.contains("Space: GO") && hints.contains("Escape: Panic"));
}

#[test]
fn a_long_name_gives_way_before_any_hint() {
    let long = format!("{}.cinq.toml", "a".repeat(55));
    let app = named(&long);
    let (name, hints) = shown(&app, COLUMNS);
    assert!(name.ends_with('…') && name.width() < long.width(), "{name}");
    assert!(
        hints.contains("Space: GO") && hints.contains("Escape: Panic") && hints.contains("help"),
        "{hints}"
    );
    // The name keeps the columns that the status and the hints leave
    let rest = " | Edit mode | 0 warnings".width();
    assert_eq!(name.width() + rest + hints.width(), COLUMNS);
}

#[test]
fn hints_give_way_once_the_name_is_down_to_its_least() {
    let app = named(&format!("{}.cinq.toml", "a".repeat(55)));
    let rest = " | Edit mode | 0 warnings".width();
    let help = "F1: help".width();
    let (name, hints) = shown(&app, rest + 16 + help);
    assert_eq!((name.width(), hints.as_str()), (16, "F1: help"));
    let (name, hints) = shown(&app, rest + 16 + help - 1);
    assert_eq!(
        (name.width(), hints.as_str()),
        (rest + 16 + help - 1 - rest, "")
    );
}
