// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of cues that send nowhere, because their outputs are gone or fail

use super::*;

#[test]
fn a_cue_whose_sends_all_name_outputs_that_are_gone_sends_nowhere() {
    let show = house_and_sub().apply(send_on(&[(2, 0.0)]));
    let stranded = |show: &Workspace| show.stranded().count();
    assert_eq!(stranded(&show), 0);
    let gone = show.clone().apply(delete_outputs(&[2]));
    assert_eq!(stranded(&gone), 1);
    assert_eq!(gone.stranded_among(show.outputs()), 0);
    assert_eq!(show.stranded_among(gone.outputs()), 1);
    // Undo brings the output and the sound back
    assert_eq!(stranded(&gone.apply(Command::Undo)), 0);
}

#[test]
fn a_cue_that_sends_nowhere_by_choice_or_by_name_does_not_count() {
    let show = house_and_sub();
    let edited = |sends| {
        show.clone()
            .apply(Command::Edit(CueId(1), Edit::Sends(sends)))
            .apply(delete_outputs(&[2]))
            .stranded()
            .count()
    };
    // No sends play on the first output, and cleared sends play nowhere on
    // purpose
    assert_eq!(edited(None), 0);
    assert_eq!(edited(Some(Sends::new())), 0);
    // One send that remains plays
    assert_eq!(edited(Some(sends(&[(1, 0.0), (2, 0.0)]))), 0);
    assert_eq!(edited(Some(sends(&[(2, 0.0)]))), 1);
}

#[test]
fn a_cue_that_sends_only_to_failing_outputs_is_silent_unless_its_file_holds_no_sound() {
    let show = workspace(vec![audio(1), video(2)])
        .apply(loaded(1, "sound.ogg", sound(10.0)))
        .apply(loaded(2, "movie.mp4", sound(10.0)));
    let lost = |show: &Workspace, id: u32| {
        let cue = show.cue(CueId(id)).expect("it exists");
        show.sends_to_failing(cue, &[MAIN])
    };
    assert_eq!((lost(&show, 1), lost(&show, 2)), (true, true));
    // Picture of a video with no sound still shows
    let picture = show.apply(loaded(2, "movie.mp4", silent(10.0)));
    assert_eq!((lost(&picture, 1), lost(&picture, 2)), (true, false));
}
