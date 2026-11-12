// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the clock of a run following its sound, on the mock backend of
//! kira, which plays ten milliseconds of sound each time a test lets it

use std::{fs, thread, time::Duration};

use kira::{
    AudioManager, AudioManagerSettings,
    backend::mock::{MockBackend, MockBackendSettings},
    sound::streaming::StreamingSoundData,
};
use tempfile::TempDir;

use super::silence;
use crate::back::{
    Command, Cue, CueId, CueList, Kind, Media, Workspace,
    audio::voice::{Voice, clipped},
    workspace::tests::{GO, loaded},
};

/// Longest that a test lets the thread of a sound decode it, which takes
/// far less
const DECODING: Duration = Duration::from_millis(300);

/// Time that the sound moves on each time a test lets the backend play
const BUFFER: f64 = 0.01;

fn video() -> Cue {
    Cue::new(CueId(1), "1", Kind::Video, "Movie")
        .file("silence.wav")
        .build()
}

fn audio() -> Cue {
    Cue::new(CueId(1), "1", Kind::Audio, "Music")
        .file("silence.wav")
        .build()
}

/// Plays the sound of the cue made by `cue` from a file of `seconds` of
/// sound, and returns how many seconds the run has played after each of the
/// `ticks` by which the clock moves on while the sound moves on ten
/// milliseconds and the run hears it, with whether the sound was heard
fn played(cue: Cue, seconds: f32, ticks: &[f64]) -> Vec<(f64, bool)> {
    played_while(cue, seconds, ticks, |_| true)
}

/// Plays as [`played`] does, but the sound moves on only at the ticks,
/// counted from zero, at which `plays` holds, as on a device that stalls
fn played_while(
    cue: Cue,
    seconds: f32,
    ticks: &[f64],
    plays: impl Fn(usize) -> bool,
) -> Vec<(f64, bool)> {
    let directory = TempDir::new().expect("a temporary directory can be made");
    fs::write(directory.path().join("silence.wav"), silence(seconds))
        .expect("the file can be written");
    let media = Media::Loaded {
        duration: 10.0,
        sound: Some(seconds),
    };
    let workspace = Workspace::new(vec![CueList::new("Main", vec![cue])])
        .directory(directory.path())
        .apply(loaded(1, "silence.wav", media))
        .apply(GO);
    let mut manager = AudioManager::<MockBackend>::new(AudioManagerSettings {
        backend_settings: MockBackendSettings { sample_rate: 8000 },
        internal_buffer_size: 80,
        ..AudioManagerSettings::default()
    })
    .expect("the mock backend starts");
    let active = workspace.active_of(CueId(1)).expect("the cue runs");
    let clip = active.sound().expect("the cue has sound");
    let data = StreamingSoundData::from_file(directory.path().join("silence.wav"))
        .expect("the sound opens");
    let sound = manager
        .play(clipped(data, clip, clip.start, 0.0))
        .expect("the sound plays");
    let voice = Voice::of(sound, active);
    thread::sleep(DECODING);
    ticks
        .iter()
        .enumerate()
        .scan(workspace, |workspace, (index, &tick)| {
            let ticked = std::mem::take(workspace).apply(Command::Tick(tick));
            if plays(index) {
                let backend = manager.backend_mut();
                backend.on_start_processing();
                backend.process();
            }
            let heard = voice.heard(ticked.active_of(CueId(1)).expect("the cue runs"));
            *workspace = heard.clone().into_iter().fold(ticked, Workspace::apply);
            let played = workspace
                .active_of(CueId(1))
                .expect("the cue runs")
                .played();
            Some((played, heard.is_some()))
        })
        .collect()
}

#[test]
fn clock_of_run_goes_to_where_its_sound_is_and_stays() {
    // The sound is a buffer behind the clock, which holds from the second
    // sync on, since the position of the sound is where it was as the last
    // buffer began
    let played = played(video(), 2.0, &[BUFFER; 60]);
    let (last, _) = played[59];
    assert!((last - 0.59).abs() < 1e-3, "{last}");
}

#[test]
fn clock_of_run_follows_a_sound_that_loops_across_its_passes() {
    let looping = Cue {
        start: 0.01.into(),
        end: 0.09.into(),
        looping: true,
        ..video()
    };
    let played = played(looping, 0.1, &[BUFFER; 40]);
    let (last, _) = played[39];
    assert!((last - 0.39).abs() < 1e-3, "{last}");
}

#[test]
fn clock_of_run_keeps_going_once_its_sound_has_ended() {
    let played = played(video(), 0.1, &[BUFFER; 14]);
    assert!(played[..8].iter().all(|(_, heard)| *heard));
    assert!(played[12..].iter().all(|(_, heard)| !*heard));
}

#[test]
fn clock_of_an_audio_cue_follows_its_sound_as_that_of_a_video_cue_does() {
    let played = played(audio(), 2.0, &[BUFFER; 60]);
    let (last, heard) = played[59];
    assert!(heard);
    assert!((last - 0.59).abs() < 1e-3, "{last}");
}

#[test]
fn clock_of_run_waits_while_its_sound_stalls() {
    // Half a second in which the device takes no sound, as one that has
    // just opened can, which the clock waits through instead of running
    // ahead, so that the run cannot end before its sound
    let played = played_while(audio(), 2.0, &[BUFFER; 100], |tick| {
        !(20..70).contains(&tick)
    });
    let (last, _) = played[99];
    assert!((last - 0.49).abs() < 0.02, "{last}");
}
