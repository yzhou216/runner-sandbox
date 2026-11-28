// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hint of the key that opens a show, which names the first step
//! while no show is open

use super::{cancel::COLUMNS, has_hint, hints_fitted, holding, memo, opened, opened_with, taken};
use crate::{
    back::fresh,
    front::{
        App,
        keymap::{Keymap, desk},
        update::Show,
    },
};

#[test]
fn the_key_of_opening_a_show_is_hinted_while_the_show_has_no_file_and_no_cues() {
    assert!(has_hint(&opened(false, false), "Control+l: open"));
}

#[test]
fn the_key_of_opening_a_show_is_not_hinted_once_a_file_or_a_cue_exists() {
    let with_file = App {
        show: Show::in_file("/shows/storm.cinq.toml"),
        ..opened(false, false)
    };
    let with_cue = App {
        workspace: fresh(vec![memo(1)]),
        ..opened(false, false)
    };
    assert!(!has_hint(&with_file, "Control+l: open"));
    assert!(!has_hint(&with_cue, "Control+l: open"));
}

#[test]
fn the_key_of_opening_a_show_is_not_hinted_where_a_field_or_the_outputs_have_the_keys() {
    let app = |held, outputs_shown| App {
        keyboard: holding(held),
        outputs_shown,
        ..opened(false, false)
    };
    assert!(!has_hint(&app(true, false), "Control+l: open"));
    assert!(!has_hint(&app(false, true), "Control+l: open"));
}

#[test]
fn the_key_of_opening_a_show_fits_a_bar_of_1024_pixels_with_and_without_a_key_to_cancel() {
    [desk(""), Keymap::default()]
        .into_iter()
        .for_each(|keymap| {
            let app = opened_with(keymap, false, false);
            let shown = hints_fitted(&app, COLUMNS - taken(&app) - 2);
            assert!(shown.contains(": open"), "{shown}");
        });
}
