// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of when the light patch shows its help on addresses

use iced::advanced::widget::{Tree, tree};

use super::{NAME_ROOM, levels::levels_now_fits, now_fits, page, shows_help};
use crate::{
    back::{Command, Cue, CueId, InstrumentId, Kind, PatchEdit, Workspace},
    front::{
        App,
        app::{Held, Keyboard},
        fixtures::{app_of, in_one_run},
        view::{char_width, widget::focus::State},
    },
};

#[test]
fn the_help_on_addresses_shows_only_while_an_address_has_the_keyboard_or_is_refused() {
    let app = |address_held| App {
        keyboard: Keyboard {
            held: Held {
                address: address_held,
                ..Held::default()
            },
            ..Keyboard::default()
        },
        ..App::default()
    };
    let refused = Some("1 to 512".to_owned());
    assert!(!shows_help(&app(false), None));
    assert!(shows_help(&app(true), None));
    assert!(shows_help(&app(false), refused.as_deref()));
}

/// Width of the inspector in the tests, which holds every column
const WIDE: f32 = 900.0;

/// Returns the state of every widget that takes the keyboard like a button,
/// in the order of the tree
fn states(tree: &mut Tree) -> Vec<&mut State> {
    let own = (tree.tag == tree::Tag::of::<State>()).then(|| tree.state.downcast_mut::<State>());
    own.into_iter()
        .chain(tree.children.iter_mut().flat_map(states))
        .collect()
}

/// Returns whether each of those has the keyboard
fn held(tree: &mut Tree) -> Vec<bool> {
    states(tree)
        .into_iter()
        .map(|state| state.focused)
        .collect()
}

/// Returns an app whose light patch holds `count` instruments
fn patched(count: usize) -> App {
    let workspace = (0..count).fold(Workspace::default(), |workspace, _| {
        workspace.apply(Command::Patch(PatchEdit::Add))
    });
    app_of(workspace)
}

#[test]
fn the_keyboard_stays_at_the_place_of_a_removed_button_for_the_row_that_slides_in() {
    // iced matches the state of widgets by position, so that the Remove
    // button of the row below takes the place and the focus of the one gone
    let before = patched(3);
    let element = page(&before, WIDE);
    let mut tree = Tree::new(&element);
    // Add instrument comes first, and the Remove button of the first row next
    states(&mut tree)[1].focused = true;
    assert_eq!(held(&mut tree), [false, true, false, false]);
    let after = patched(2);
    tree.diff(page(&after, WIDE));
    assert_eq!(held(&mut tree), [false, true, false]);
}

#[test]
fn the_now_column_gives_way_to_a_name_that_would_keep_too_little_room() {
    let kept = in_one_run(0..2000, now_fits);
    assert!(!now_fits(0.0) && now_fits(WIDE));
    // The name keeps its room beside the column, with nothing else to give
    assert!(kept[0] > NAME_ROOM * char_width());
}

#[test]
fn the_now_column_of_the_levels_gives_way_where_the_patch_keeps_it_and_no_sooner() {
    let kept = in_one_run(0..2000, levels_now_fits);
    assert!(!levels_now_fits(0.0) && levels_now_fits(WIDE));
    // One column fewer than the patch, which has the Remove button too
    assert!(kept[0] < in_one_run(0..2000, now_fits)[0]);
}

#[test]
fn the_levels_list_an_instrument_that_the_patch_lacks_after_those_it_has() {
    let patch = patched(2).workspace.patch().to_vec();
    let cue = Cue::new(CueId(1), "1", Kind::Light, "Light")
        .lights(
            [(InstrumentId(99), 50.0), (InstrumentId(1), 10.0)]
                .into_iter()
                .collect(),
        )
        .build();
    let names = super::levels::rows(&patch, &cue)
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(names, ["1", "2", "99 (unpatched)"]);
}

#[test]
fn empty_levels_tell_which_tab_adds_an_instrument() {
    assert!(super::levels::no_instruments().ends_with("on the Light Patch tab"));
}
