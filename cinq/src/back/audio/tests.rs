// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the audio of cues and helpers that they share, none of which
//! needs an audio device

mod ahead;
mod cutting;
mod devamping;
mod device;
mod fakes;
mod leaving;
mod opener;
mod opening;
mod patch;
mod preloading;
mod retry;
mod routing;
mod starting;
mod voice;
mod wording;

use std::{fs, path::Path, thread, time::Duration};

use kira::{
    AudioManager, AudioManagerSettings,
    backend::mock::{MockBackend, MockBackendSettings},
};
use tempfile::TempDir;

use super::{
    Audio, READY_AHEAD,
    sound::preload::{Ask, Preloaded, Preloader, Wanted},
    upcoming,
};
use crate::back::{Clip, CueId, Workspace};

/// Returns the manager of kira on its mock backend, which plays one buffer of
/// `frames` frames at `rate` frames a second each time a test lets it
fn mock_manager(rate: u32, frames: usize) -> AudioManager<MockBackend> {
    AudioManager::new(AudioManagerSettings {
        backend_settings: MockBackendSettings { sample_rate: rate },
        internal_buffer_size: frames,
        ..AudioManagerSettings::default()
    })
    .expect("the mock backend starts")
}

fn upcoming_cues(workspace: &Workspace) -> Vec<u32> {
    cues_within(workspace, READY_AHEAD)
}

fn cues_within(workspace: &Workspace, ahead: f64) -> Vec<u32> {
    upcoming(workspace, ahead)
        .into_iter()
        .map(|ask| ask.cue.0)
        .collect()
}

/// Returns the ask of cue 1 for `wanted`, at no change of level
fn ask(wanted: &Wanted) -> Ask {
    Ask {
        cue: CueId(1),
        wanted: wanted.clone(),
        level: 0.0,
    }
}

/// Returns `seconds` of silence as a mono WAV file at 8000 frames a second
pub(in crate::back) fn silence(seconds: f32) -> Vec<u8> {
    silence_at(seconds, 8000)
}

pub(in crate::back) fn silence_at(seconds: f32, rate: u32) -> Vec<u8> {
    silence_in(seconds, rate, 1)
}

/// Returns `seconds` of silence as a WAV file in 16 bits
fn silence_in(seconds: f32, rate: u32, channels: u16) -> Vec<u8> {
    let width = 2 * channels;
    let data = (seconds * rate as f32) as u32 * u32::from(width);
    [
        b"RIFF".as_slice(),
        &(36 + data).to_le_bytes(),
        b"WAVE",
        b"fmt ",
        &16_u32.to_le_bytes(),
        &1_u16.to_le_bytes(),
        &channels.to_le_bytes(),
        &rate.to_le_bytes(),
        &(rate * u32::from(width)).to_le_bytes(),
        &width.to_le_bytes(),
        &16_u16.to_le_bytes(),
        b"data",
        &data.to_le_bytes(),
        &vec![0; data as usize],
    ]
    .concat()
}

/// Returns the whole of a tenth of a second of sound in `path` as wanted
fn whole(path: &Path) -> Wanted {
    Wanted {
        path: path.to_owned(),
        clip: Clip {
            start: 0.0,
            end: 0.1,
            looping: None,
        },
    }
}

fn silent_file() -> (TempDir, Wanted) {
    let directory = TempDir::new().expect("a temporary directory can be made");
    let file = directory.path().join("silence.wav");
    fs::write(&file, silence(0.1)).expect("the file can be written");
    (directory, whole(&file))
}

/// Calls `poll` every five milliseconds until it returns something, five
/// seconds at most, since a thread of the test is to answer
fn waiting<T>(mut poll: impl FnMut() -> Option<T>) -> Option<T> {
    (0..1000).find_map(|_| {
        poll().or_else(|| {
            thread::sleep(Duration::from_millis(5));
            None
        })
    })
}

/// Waits until `done` holds of `preloader`, taking in what its thread made
fn collecting(preloader: &mut Preloader, done: impl Fn(&Preloader) -> bool) -> Option<()> {
    waiting(|| {
        preloader.collect();
        done(preloader).then_some(())
    })
}

/// Returns what `preloader` made for `cue`, once its thread has made it
fn made(
    preloader: &mut Preloader,
    cue: CueId,
    wanted: &Wanted,
) -> Option<Result<Preloaded, String>> {
    waiting(|| {
        preloader.collect();
        preloader.take(cue, wanted)
    })
}

/// Waits until `done` holds, taking in the answers of the threads of the
/// devices as a sync does, and fails the test after five seconds
fn until(audio: &mut Audio, done: impl Fn(&Audio) -> bool) {
    waiting(|| {
        audio.arrived();
        done(audio).then_some(())
    })
    .expect("the threads of the devices answered within five seconds");
}

/// Waits until the threads of the devices have answered every job
fn settle(audio: &mut Audio) {
    until(audio, |audio| !audio.opener.busy());
}
