// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where the sound of a run that a devamp cue lets go on starts,
//! what is made ready for it, and how far kira's decoder runs ahead of the
//! sound, which is why the voice of a run waits to take away its loop

use std::{fs, iter, thread, time::Duration};

use kira::{
    AudioManager,
    backend::mock::MockBackend,
    sound::{
        EndPosition, FromFileError, Region,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
};
use tempfile::TempDir;

use super::{
    collecting,
    devamping::{RATE, vamp},
    mock_manager, silence_at, upcoming,
    voice::target,
    waiting as waiting_for,
};
use crate::back::{
    Command, CueId, READY_AHEAD, Workspace,
    audio::sound::{
        mixer::Mixer,
        preload::{Ask, Preloader},
        voice::Voice,
    },
    workspace::tests::GO,
};

/// Starts the sound of the cue that [`vamp`] fires, which loops from 1 to 2
/// seconds into a file of 4, on a mixer that [`Mix`](super::super::sound::mixer::Mix)
/// renders, as a run that has played `played` seconds, and returns the voice
/// with the seconds of sound that it still has after `rendered` seconds.
/// Kira's decoder thread may run behind the mixer on a busy machine, and a
/// sound without decoded frames waits instead of moving on, so a sound that
/// `ends` is rendered on until it has stopped, for five seconds at most.
fn opened_at(played: f64, rendered: f64, ends: bool) -> (bool, bool) {
    let (directory, workspace) = vamp(8000, 4.0, (1.0, 2.0), 0.0);
    let workspace = workspace
        .apply(Command::Tick(played.min(0.4)))
        .apply(GO)
        .apply(Command::Tick((played - 0.4).max(0.0)));
    let active = workspace.active_of(CueId(1)).expect("the cue runs");
    let clip = active.sound().expect("the cue has sound");
    let (mut mixer, mut mix) = Mixer::bench(2, 2, 8000, 16);
    let targets = vec![target("a", &mut mixer)];
    let voice = Voice::open(targets, &directory.path().join("silence.wav"), active, clip)
        .expect("the sound starts");
    let mut render = || mix.render(&mut [0.0_f32; 512]);
    (0..(rendered * 8000.0 / 256.0) as usize).for_each(|_| {
        render();
        thread::sleep(Duration::from_millis(2));
    });
    let heard = || voice.heard(active, |_| true).is_some();
    if ends {
        waiting_for(|| {
            render();
            (!heard()).then_some(())
        });
    }
    (voice.looped(), heard())
}

#[test]
fn a_voice_opened_past_the_last_pass_has_no_loop_and_plays_to_the_end() {
    // Released at 2 seconds, so that the last pass begins at 1 second: the
    // sound plays from 1.2 seconds to the end of the file at 4 seconds
    assert_eq!(opened_at(1.2, 3.2, true), (false, false));
}

#[test]
fn a_voice_opened_before_the_last_pass_keeps_its_loop() {
    assert_eq!(opened_at(0.4, 3.2, false), (true, true));
}

#[test]
fn a_run_devamped_in_its_pre_wait_still_takes_the_sound_made_ready() {
    let (_directory, waiting) = vamp(8000, 2.5, (0.5, 1.5), 1.0);
    let asks = |workspace: &Workspace| -> Vec<Ask> {
        upcoming(workspace, READY_AHEAD)
            .into_iter()
            .filter(|ask| ask.cue == CueId(1))
            .collect()
    };
    let mut preloader = Preloader::new();
    preloader.want(asks(&waiting));
    collecting(&mut preloader, |preloader| preloader.ready(CueId(1)))
        .expect("the sound is made ready");
    let devamped = waiting.apply(GO);
    let after = asks(&devamped);
    let released = after[0].wanted.clone();
    assert!(
        released
            .clip
            .looping
            .is_some_and(|section| section.released.is_some()),
        "the devamp cue has let the run go on"
    );
    preloader.want(after);
    assert!(
        preloader.ready(CueId(1)),
        "the sound is still wanted once the run is let go on"
    );
    assert!(
        matches!(preloader.take(CueId(1), &released), Some(Ok(_))),
        "the run takes it"
    );
}
/// Plays `manager` one buffer on and returns where the sound is
fn advanced(
    manager: &mut AudioManager<MockBackend>,
    sound: &StreamingSoundHandle<FromFileError>,
) -> f64 {
    let backend = manager.backend_mut();
    backend.on_start_processing();
    backend.process();
    thread::sleep(Duration::from_millis(1));
    sound.position()
}

/// Plays the sound of a loop from 1 to 3 seconds in a file of 4, from 2.4
/// seconds, on the mock backend, and takes the loop away once the sound is
/// `frames` frames short of the end of the loop, with the decoder as far
/// ahead as it runs.  Returns how many times the sound went back.
fn wraps_after_taking_the_loop_away(frames: usize) -> usize {
    let mut manager = mock_manager(RATE, 120);
    let directory = TempDir::new().expect("a temporary directory can be made");
    let file = directory.path().join("silence.wav");
    fs::write(&file, silence_at(4.0, RATE)).expect("the file can be written");
    let mut sound = manager
        .play(
            StreamingSoundData::from_file(&file)
                .expect("the sound opens")
                .loop_region(Region {
                    start: 1.0.into(),
                    end: EndPosition::Custom(3.0.into()),
                })
                .start_position(2.4),
        )
        .expect("the sound plays");
    thread::sleep(Duration::from_millis(300));
    let target = 3.0 - frames as f64 / f64::from(RATE);
    let before: Vec<f64> = iter::from_fn(|| Some(advanced(&mut manager, &sound)))
        .take_while(|position| *position < target)
        .collect();
    thread::sleep(Duration::from_millis(50));
    sound.set_loop_region(None);
    let after: Vec<f64> = (0..200).map(|_| advanced(&mut manager, &sound)).collect();
    before
        .iter()
        .chain(&after)
        .collect::<Vec<_>>()
        .windows(2)
        .filter(|pair| *pair[1] < *pair[0] - 0.5)
        .count()
}

#[test]
fn a_sound_leaves_its_loop_only_at_a_pass_end_that_its_decoder_has_not_passed() {
    // Kira decodes 16384 frames ahead of the sound, which is 0.34 seconds
    // here: taking the loop away less than that before its end is too late
    // to stop the pass, and a little more than that is early enough
    assert_eq!(wraps_after_taking_the_loop_away(10_000), 1);
    assert_eq!(wraps_after_taking_the_loop_away(20_000), 0);
}
