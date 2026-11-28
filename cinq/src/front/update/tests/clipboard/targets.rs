// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the targets of fade and stop cues that are pasted

use super::*;

/// Returns the state of a show of audio cue 1, fade cue 2 that targets it,
/// and audio cue 3, with cue 1 selected
fn fading() -> App {
    let fade = Cue::new(CueId(2), "2", Kind::Fade, "Fade")
        .target(CueId(1))
        .build();
    one_list(vec![audio(1), fade, audio(3)], Some(CueId(1)))
}

/// Returns the identity and target of each cue of the first list
fn targets(app: &App) -> Vec<(u32, Option<u32>)> {
    crate::back::targets(&app.workspace)
}

#[test]
fn a_fade_cut_with_its_target_targets_the_new_copy_where_it_is_pasted() {
    let app = after(
        fading(),
        [
            Message::Cut(2),
            Message::Select(CueId(3)),
            Message::Paste(Place::After, 1),
        ],
    );
    assert_eq!(targets(&app), [(3, None), (4, None), (5, Some(4))]);
}

#[test]
fn a_fade_copied_with_its_target_targets_the_new_copy_and_the_old_one_stays() {
    let app = after(
        fading(),
        [
            Message::Copy(2),
            Message::Select(CueId(3)),
            Message::Paste(Place::After, 1),
        ],
    );
    let expected = [(1, None), (2, Some(1)), (3, None), (4, None), (5, Some(4))];
    assert_eq!(targets(&app), expected);
}

#[test]
fn each_paste_of_a_count_targets_the_copy_of_its_own_round() {
    let app = after(
        fading(),
        [
            Message::Copy(2),
            Message::Select(CueId(3)),
            Message::Paste(Place::After, 2),
        ],
    );
    assert_eq!(
        targets(&app)[3..],
        [(4, None), (5, Some(4)), (6, None), (7, Some(6))]
    );
}

#[test]
fn a_fade_copied_without_its_target_keeps_it_while_it_is_there() {
    let app = after(
        fading(),
        [
            Message::Select(CueId(2)),
            Message::Copy(1),
            Message::Paste(Place::After, 1),
        ],
    );
    assert_eq!(targets(&app)[2], (4, Some(1)));
}

#[test]
fn a_fade_cut_or_copied_over_a_motion_remaps_its_target_as_a_count_does() {
    use crate::front::keymap::Action::{Copy, Cut, Next};
    let pasted = |operator| {
        after(
            operators::across(fading(), operator, Next, None),
            [Message::Select(CueId(3)), Message::Paste(Place::After, 1)],
        )
    };
    assert_eq!(targets(&pasted(Cut)), [(3, None), (4, None), (5, Some(4))]);
    let copied = [(1, None), (2, Some(1)), (3, None), (4, None), (5, Some(4))];
    assert_eq!(targets(&pasted(Copy)), copied);
    // Undo takes the paste and leaves the cut
    assert_eq!(targets(&typing(pasted(Cut), "u")), [(3, None)]);
}
