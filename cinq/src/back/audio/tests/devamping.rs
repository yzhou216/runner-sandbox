// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the sound of a cue that loops part of its clip, and of a devamp
//! cue letting it go on, through kira itself: its streaming decoder runs
//! ahead of the sound, which is what the voice of a run has to allow for.
//! The sounds play in the mock backend of kira at 48000 frames a second,
//! which plays one tick of sound each time a test lets it.
mod devamped;
mod looping;

use std::{fs, thread, time::Duration};

use kira::{AudioManager, backend::mock::MockBackend, sound::streaming::StreamingSoundData};
use tempfile::TempDir;

use super::{mock_manager, silence_at, voice::audio};
use crate::back::{
    Active, Command, Cue, CueId, Workspace,
    audio::sound::voice::{Voice, clipped},
    workspace::tests::{GO, devamp_cue, fresh, loaded, sound},
};

pub(super) const RATE: u32 = 48_000;

/// Seconds that the workspace moves on in each step of a test
const TICK: f64 = 0.025;

/// Frames in one step, which is one tick at `RATE`
const FRAMES: usize = 1200;

/// Time that the thread of a sound gets to decode in each step, far more
/// than it needs
const DECODING: Duration = Duration::from_millis(3);

/// Returns a directory with a file of `seconds` of silence at `rate`, and
/// the workspace in which GO has fired an audio cue that loops from `from`
/// to `to` in that file, with a devamp cue after it
pub(super) fn vamp(
    rate: u32,
    seconds: f32,
    (from, to): (f32, f32),
    pre_wait: f32,
) -> (TempDir, Workspace) {
    let directory = TempDir::new().expect("a temporary directory can be made");
    fs::write(
        directory.path().join("silence.wav"),
        silence_at(seconds, rate),
    )
    .expect("the file can be written");
    let cue = Cue {
        pre_wait: pre_wait.into(),
        looping: true,
        loop_start: from.into(),
        loop_end: to.into(),
        ..audio()
    };
    let workspace = fresh(vec![cue, devamp_cue(2, 1)])
        .directory(directory.path())
        .apply(loaded(1, "silence.wav", sound(seconds)))
        .apply(GO);
    (directory, workspace)
}

/// What a test sees after a step
#[derive(Clone, Copy)]
struct Seen {
    /// Where in its file the sound is
    position: f64,
    /// Seconds that the run has played, while it runs
    played: Option<f64>,
    /// Whether the sound still plays
    heard: bool,
    /// Whether the voice tells that the sound stopped early
    cut: bool,
}

/// Run and its sound in a mock backend
struct Rig {
    manager: AudioManager<MockBackend>,
    workspace: Workspace,
    voice: Voice,
}

impl Rig {
    /// Starts the sound of cue 1, which runs in `workspace`, as one opened
    /// where the run is
    fn start(directory: &TempDir, workspace: Workspace) -> Self {
        let mut manager = mock_manager(RATE, FRAMES);
        let active = workspace.active_of(CueId(1)).expect("the cue runs");
        let clip = active.sound().expect("the cue has sound");
        let data = StreamingSoundData::from_file(directory.path().join("silence.wav"))
            .expect("the sound opens");
        let sound = manager
            .play(clipped(data, clip, active.played(), 0.0))
            .expect("the sound plays");
        let voice = Voice::of(sound, active);
        thread::sleep(Duration::from_millis(300));
        Self {
            manager,
            workspace,
            voice,
        }
    }

    /// Moves the clock and the sound on one tick and syncs the voice as the
    /// audio does
    fn stepped(self) -> (Self, Seen) {
        let Self {
            mut manager,
            workspace,
            voice,
        } = self;
        let ticked = workspace.apply(Command::Tick(TICK));
        let backend = manager.backend_mut();
        backend.on_start_processing();
        backend.process();
        thread::sleep(DECODING);
        let active = ticked.active_of(CueId(1)).copied();
        let voice = active.iter().fold(voice, Voice::followed);
        let heard = active
            .as_ref()
            .and_then(|active| voice.heard(active, |_| true));
        let cut = active.as_ref().is_some_and(|active| {
            let clip = active.sound().expect("the cue has sound");
            let ends = ticked.cue(CueId(1)).and_then(|cue| cue.sound_ends(clip));
            ends.is_some_and(|ends| voice.cut_short(clip, ends, |_| true))
        });
        let workspace = heard.clone().into_iter().fold(ticked, Workspace::apply);
        let seen = Seen {
            position: voice.position().expect("the voice has a sound"),
            played: workspace.active_of(CueId(1)).map(Active::played),
            heard: heard.is_some(),
            cut,
        };
        (
            Self {
                manager,
                workspace,
                voice,
            },
            seen,
        )
    }

    /// Steps `steps` times, and before step `index` lets `act` change the
    /// workspace
    fn run(self, steps: usize, act: impl Fn(usize, Workspace) -> Workspace) -> (Self, Vec<Seen>) {
        (0..steps).fold((self, Vec::new()), |(rig, seen), index| {
            let (rig, now) = Self {
                workspace: act(index, rig.workspace),
                ..rig
            }
            .stepped();
            (rig, seen.into_iter().chain([now]).collect())
        })
    }
}

/// Returns an action that fires GO before step `step` only
fn go_at(step: usize) -> impl Fn(usize, Workspace) -> Workspace {
    move |index, workspace| {
        if index == step {
            workspace.apply(GO)
        } else {
            workspace
        }
    }
}

/// Returns how many times the sound went back, as at the end of a loop pass
fn wraps(seen: &[Seen]) -> usize {
    seen.windows(2)
        .filter(|pair| pair[1].position < pair[0].position - 0.5)
        .count()
}

/// Returns the most that the clock of the run is off the time that the sound
/// has played
fn drift(seen: &[Seen]) -> f64 {
    seen.iter()
        .enumerate()
        .filter_map(|(index, seen)| Some((seen.played? - (index + 1) as f64 * TICK).abs()))
        .fold(0.0, f64::max)
}
