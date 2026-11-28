// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of telling a sound that stops before the end of its file, as one
//! does whose file is cut short, on the mock backend of kira, which plays
//! ten milliseconds of sound each time a test lets it.  The decoder of a
//! file whose data lies short of its header fails at once, so that its sound
//! stops as it starts.

use std::thread;

use super::voice::{DECODING, audio, fired_promising, sounding, target, video};
use crate::back::{
    Active, Cue, CueId,
    audio::{
        patch::Key,
        sound::{mixer::Mixer, voice::Voice},
    },
    workspace::tests::looped,
};

/// Plays `buffers` buffers of the sound of `cue` from a file that holds
/// `seconds` of sound where its header promises `promised`, and returns
/// whether the voice then tells that the sound stopped early
fn cut(cue: Cue, seconds: f32, promised: f32, buffers: usize) -> bool {
    let (directory, workspace) = fired_promising(cue, seconds, promised);
    let (mut manager, voice) = sounding(&directory, &workspace);
    (0..buffers).for_each(|_| {
        let backend = manager.backend_mut();
        backend.on_start_processing();
        backend.process();
    });
    let cue = workspace.cue(CueId(1)).expect("the cue is there");
    let clip = workspace
        .active_of(CueId(1))
        .and_then(Active::sound)
        .expect("the cue has sound");
    let ends = cue.sound_ends(clip).expect("the cue has sound");
    voice.cut_short(clip, ends, |_| true)
}

#[test]
fn sound_is_cut_short_only_when_it_stops_with_more_than_a_second_of_its_file_ahead() {
    // Rows: cue, seconds in the file, seconds that its header promises, and
    // buffers played
    let cases = [
        ("over a second ahead", audio(), 0.5, 2.0, 80, true),
        ("within a second of its end", audio(), 0.2, 0.8, 80, false),
        ("plays on", audio(), 5.0, 5.0, 20, false),
        ("plays to the end of its file", audio(), 0.5, 0.5, 80, false),
        // However little lies ahead
        ("loop that stops", looped(audio()), 0.2, 0.5, 80, true),
        // Picture plays for ten seconds, which the sound track need not
        ("sound track ends first", video(), 0.5, 0.5, 80, false),
    ];
    cases
        .into_iter()
        .for_each(|(name, cue, seconds, promised, buffers, early)| {
            assert_eq!(cut(cue, seconds, promised, buffers), early, "{name}");
        });
}

#[test]
fn sound_on_a_device_that_does_not_play_is_not() {
    let (directory, workspace) = fired_promising(audio(), 0.5, 2.0);
    let active = workspace.active_of(CueId(1)).expect("the cue runs");
    let clip = active.sound().expect("the cue has sound");
    let ends = workspace
        .cue(CueId(1))
        .and_then(|cue| cue.sound_ends(clip))
        .expect("the cue has sound");
    let (mut mixer, mut mix) = Mixer::bench(2, 2, 8000, 16);
    let targets = vec![target("a", &mut mixer)];
    let voice = Voice::open(targets, &directory.path().join("silence.wav"), active, clip)
        .expect("the device takes the sound");
    thread::sleep(DECODING);
    (0..3).for_each(|_| mix.render(&mut [0.0_f32; 512]));
    let on = |key: &Key| key.as_deref() == Some("a");
    assert!(voice.cut_short(clip, ends, on));
    assert!(!voice.cut_short(clip, ends, |_| false));
}
