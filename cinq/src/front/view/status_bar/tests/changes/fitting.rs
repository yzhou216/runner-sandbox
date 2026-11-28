// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how a notice of a change fits in the room that the hints leave

use super::*;

#[test]
fn what_a_change_did_to_the_cues_comes_first_and_stays_whole_beside_long_names() {
    let base = rain()
        .apply(rename(2, "Dante Virtual Soundcard Left Main Speaker"))
        .apply(rename(1, "Another long name"));
    // The bar of 1024 pixels with the mark of unsaved changes, which takes ten
    // columns of the status
    let in_bar = |workspace| told(workspace, COLUMNS - 10);
    let deleted = base
        .clone()
        .apply(Command::DeleteOutputs(vec![OutputId(1)]));
    assert_eq!(
        in_bar(deleted.clone()),
        "Cues without sends play on Dante Vir…, deleted… | u: undo"
    );
    assert_eq!(
        in_bar(deleted.clone().apply(Command::Undo)),
        "Cues without sends play on Another l… | Control+r: redo"
    );
    assert_eq!(
        in_bar(deleted.apply(Command::Undo).apply(Command::Redo)),
        "Cues without sends play on Dante Vir…, redid… | u: undo"
    );
    let moved = Command::MoveOutput {
        output: OutputId(1),
        by: 1,
    };
    assert_eq!(
        in_bar(base.clone().apply(moved)),
        "Cues without sends play on Dante Vir…, moved… | u: undo"
    );
    let added = Command::add_output(0, 1);
    assert_eq!(
        in_bar(base.clone().apply(added)),
        "Cues without sends play on Output 1, added Output 1 | u: undo"
    );
    // A change that leaves the first output alone tells itself, with its names
    // short
    let calm = Command::EditOutput(
        OutputId(1),
        OutputEdit::Channels(crate::back::Channels::Mono(1)),
    );
    assert_eq!(
        in_bar(base.apply(calm)),
        "Changed the channels of Another l… | u: undo | ?: help"
    );
}

#[test]
fn the_notice_of_a_cut_keeps_what_it_did_to_the_cues_whole_and_gives_up_the_key_to_paste() {
    let before = rain().apply(rename(2, "Sub"));
    let taken = before.outputs()[..1].to_vec();
    let workspace = before.clone().apply(Command::CutOutputs(vec![OutputId(1)]));
    let facts = format::consequences(&workspace, before.outputs(), workspace.outputs());
    let app = App {
        notices: Notices {
            clipped: Some(Clipped {
                told: format::clipped_outputs("cut", &taken, facts),
                cut: true,
            }),
            ..Notices::default()
        },
        workspace,
        ..App::default()
    };
    // The bar of 1024 pixels, with the mark of unsaved changes and a warning
    // in the status
    let shown = |room| name::shown(&app, room).1;
    assert_eq!(
        shown(COLUMNS - 10),
        "Cues without sends play on Sub, cut Main | p: paste | u: undo"
    );
    // Room for the notice and the key of undo alone, as a longer status leaves
    assert_eq!(
        shown(COLUMNS - 20),
        "Cues without sends play on Sub, cut Main | u: undo"
    );
}

#[test]
fn an_undo_that_moves_the_first_output_back_tells_where_cues_play_without_a_dangling_ellipsis() {
    let deleted = rain().apply(Command::DeleteOutputs(vec![OutputId(1)]));
    let undone = deleted.apply(Command::Undo);
    assert_eq!(
        told(undone.clone(), COLUMNS),
        "Cues without sends play on Main, undid: deleted Main | Control+r: redo"
    );
    // With the mark of unsaved changes in the status, what was undone gives way
    // to the words that tell where the cues play, and then goes altogether,
    // leaving no ellipsis after a name that is whole
    assert_eq!(
        told(undone.clone(), COLUMNS - 10),
        "Cues without sends play on Main, undid… | Control+r: redo"
    );
    assert_eq!(
        told(undone, COLUMNS - 20),
        "Cues without sends play on Main | Control+r: redo"
    );
}

#[test]
fn a_notice_that_fits_in_no_room_ends_in_an_ellipsis_at_the_edge_of_the_room() {
    let base = rain().apply(rename(1, "House speaker pair 02"));
    let app = App {
        workspace: base.apply(Command::CutOutputs(vec![OutputId(1)])),
        outputs_shown: true,
        ..App::default()
    };
    // Rooms narrower than the notice with its keys cut to their least, which
    // the cut at a word does not reach, and than the one key that stays
    // where the notice is left out
    (4..7).for_each(|room| {
        let hints = hints_in(&app, room);
        assert_eq!(hints.width(), room, "{hints}");
        assert!(hints.ends_with('…'), "{hints}");
    });
}

#[test]
fn a_notice_in_the_least_rooms_never_overruns_them() {
    let long = "House-speaker-pair-02-with-a-very-long-single-word";
    let base = rain().apply(rename(1, long));
    let app = App {
        workspace: base.apply(Command::CutOutputs(vec![OutputId(1)])),
        outputs_shown: true,
        ..App::default()
    };
    (0..4).for_each(|room| {
        let hints = hints_in(&app, room);
        assert!(hints.width() <= room, "room {room}: {hints:?}");
    });
}
