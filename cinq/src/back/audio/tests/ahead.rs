// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which sounds are made ready ahead, by the cues that start soon

use std::path::PathBuf;

use super::{READY_AHEAD, cues_within, upcoming, upcoming_cues};
use crate::back::{
    Command,
    workspace::tests::{
        GO, audio, continued, continuing_after, loaded, pre_waiting, silent, video, workspace,
    },
};

#[test]
fn sounds_made_ready_are_those_of_the_next_go_and_of_runs_yet_to_sound() {
    let workspace =
        workspace(vec![continuing_after(1, 2.0), audio(2), audio(3)]).directory("/show");
    assert_eq!(upcoming_cues(&workspace), [1, 2]);
    let fired = workspace.apply(GO);
    assert_eq!(upcoming_cues(&fired), [2, 3]);
    let wanted = upcoming(&fired, READY_AHEAD).remove(0).wanted;
    assert_eq!(wanted.path, PathBuf::from("/show/sound.ogg"));
}

#[test]
fn sound_of_a_video_cue_is_made_ready_unless_its_file_holds_none() {
    let workspace = workspace(vec![continued(video(1)), continued(video(2))]);
    assert_eq!(upcoming_cues(&workspace), [1, 2]);
    let without_sound = workspace.apply(loaded(2, "movie.mp4", silent(10.0)));
    assert_eq!(upcoming_cues(&without_sound), [1]);
}

#[test]
fn sounds_are_made_ready_only_when_they_start_within_the_window() {
    // Cues 2 and 3 start together, 4 moves on past its pre-wait, so the
    // five start 0, 4, 4, 11, and 16 seconds after GO
    let chain = workspace(vec![
        continuing_after(1, 4.0),
        continuing_after(2, 0.0),
        continuing_after(3, 6.0),
        pre_waiting(1.0, continuing_after(4, 5.0)),
        audio(5),
    ]);
    let fired = chain.clone().apply(GO);
    let later = fired.clone().apply(Command::Tick(2.0));
    let cases = [
        ("at the instant of GO only", &chain, 0.0, vec![1]),
        ("sounds that start together", &chain, 4.0, vec![1, 2, 3]),
        ("none outside it", &chain, 10.0, vec![1, 2, 3]),
        ("on its edge", &chain, 11.0, vec![1, 2, 3, 4]),
        ("fired, with the first sounding", &fired, 10.0, vec![2, 3]),
        ("fired, as the time passes", &later, 10.0, vec![2, 3, 4]),
    ];
    cases
        .into_iter()
        .for_each(|(name, workspace, ahead, cues)| {
            assert_eq!(cues_within(workspace, ahead), cues, "{name}");
        });
}
