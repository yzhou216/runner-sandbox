// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sound output, where the back end has side effects
//!
//! [`Audio::sync`] runs after every change to the workspace.  The workspace
//! stays the only record of what should be heard, so a sound that cannot
//! start goes back to it as a file that cannot be played.  A thread of its
//! own makes ready the sounds that runs may start next, so that GO waits on
//! no file.  The sound of a video file plays from the file's sound track,
//! along with the silence that AAC encoders put around the sound, since the
//! decoder reads no edit list, which marks that silence for players to
//! skip.  Each cue plays on the outputs that it sends to, which play on
//! channels of audio devices, as [`patch`] tells, and a [`mixer`] of
//! cinq's own plays each device, since kira mixes in stereo alone.

mod device;
mod fault;
mod feed;
mod mixer;
mod opening;
mod patch;
mod play;
mod preload;
#[cfg(test)]
mod tests;
mod voice;

use std::{collections::HashMap, path::Path, time::Instant};

use itertools::Itertools;
use kira::sound::{FromFileError, streaming::StreamingSoundData};

use super::{CueId, OutputId, Workspace, load::unreadable};
pub use device::devices;
pub use fault::Fault;
use mixer::Mixer;
use patch::Patch;
use preload::{Ask, Preloader, Wanted};
use voice::{Mixers, Voice};

/// Endings of the names of the sound files that the sound output plays,
/// which the features of kira in `Cargo.toml` decide
pub const SOUND_FILES: [&str; 5] = ["flac", "m4a", "mp3", "ogg", "wav"];

/// Seconds before a sound starts that it is made ready, unless the
/// configuration file sets another number
pub const READY_AHEAD: f64 = 10.0;

pub struct Audio {
    /// ID of the device that outputs on the default device play on, or none
    /// for the default device of the system
    default: Option<String>,
    /// Where the outputs play, as the mixers opened for them
    patch: Patch,
    mixers: Mixers,
    voices: HashMap<CueId, Voice>,
    preloader: Preloader,
    /// When the devices that failed were last tried again
    tried: Instant,
}

impl Audio {
    /// Plays the outputs on the default device on the device whose ID is
    /// `default`, or on the default device of the system without one.  The
    /// devices open as the first sync learns the outputs.  Without a device
    /// to open, the workspace still runs and files still load, but nothing
    /// is heard.
    pub fn open(default: Option<&str>) -> Self {
        Self {
            default: default.map(str::to_owned),
            patch: Patch::default(),
            mixers: HashMap::new(),
            voices: HashMap::new(),
            preloader: Preloader::new(),
            tried: Instant::now(),
        }
    }

    /// Plays the outputs on the default device on the device whose ID is
    /// `default`, unless they play there already.  The sounds that play end
    /// with the old devices, and the next sync starts them on the new ones
    /// from where they are.  The sounds made ready ahead are made again, or
    /// a cue that plays and that the next GO fires again would start over
    /// from the sound made ready for that GO.
    pub fn route(&mut self, default: Option<&str>) {
        if self.default.as_deref() != default {
            *self = Self::open(default);
        }
    }

    /// Returns why each output that cannot play cannot: its device is not
    /// there, cannot be opened, or has stopped, or its channels lie past
    /// those of the device
    pub fn problems(&self) -> Vec<(OutputId, Fault)> {
        self.patch
            .outputs()
            .iter()
            .filter_map(|patched| {
                let fault = match self.mixers.get(&patched.key)? {
                    Err(fault) => fault.clone(),
                    Ok(mixer) => mixer.failure().map(Fault::Stopped).or_else(|| {
                        let channels = mixer.channels();
                        (patched.channels.needs() > channels).then_some(Fault::Channels(channels))
                    })?,
                };
                Some((patched.id, fault))
            })
            .collect()
    }

    /// Makes the sound match the workspace, and returns the workspace with
    /// the files whose sound could not start.  The sounds that runs may
    /// start soon are made ready once those that start now have taken
    /// theirs, unless there is no audio device to hear them on.
    pub fn sync(&mut self, workspace: Workspace, ahead: f64) -> Workspace {
        self.patched(Patch::of(workspace.outputs(), &self.default));
        self.revived();
        self.preloader.collect();
        self.mixers
            .values_mut()
            .filter_map(|mixer| mixer.as_mut().ok())
            .for_each(Mixer::collect);
        let failures = self.play(&workspace);
        if self.mixers.values().any(Result::is_ok) {
            self.preloader.want(upcoming(&workspace, ahead));
        }
        failures.into_iter().fold(workspace, Workspace::apply)
    }
}

/// Returns the sounds that runs may start within `ahead` seconds: first
/// those of the runs that GO has fired and that have yet to sound, which
/// start soonest, then those of the next GO of each list.  Each sound is
/// judged by its own start, so that sounds that start together are never
/// split, and those that start at the instant of GO are always among them.
fn upcoming(workspace: &Workspace, ahead: f64) -> Vec<Ask> {
    let fired = workspace.active().iter().filter_map(|active| {
        let clip = active.pending().filter(|_| active.starts_in() <= ahead)?;
        let path = workspace.path(workspace.cue(active.id)?)?;
        Some(Ask {
            cue: active.id,
            wanted: Wanted { path, clip },
            level: active.level,
        })
    });
    let next = workspace.lists().iter().flat_map(|list| {
        workspace.next_go(list).filter_map(|(cue, delay)| {
            let starts = f64::from(delay) + f64::from(cue.pre_wait.value());
            let clip = cue.clip().filter(|_| cue.sounds() && starts <= ahead)?;
            Some(Ask {
                cue: cue.id,
                wanted: Wanted {
                    path: workspace.path(cue)?,
                    clip,
                },
                level: cue.level,
            })
        })
    });
    fired.chain(next).unique_by(|ask| ask.cue).collect()
}

/// Reads how many seconds the sound of a file plays, or why it cannot be
/// played.  This opens the file and reads its format without decoding the
/// sound.
pub(super) fn probe(file: &Path) -> Result<f32, String> {
    StreamingSoundData::from_file(file)
        .map(|data| data.duration().as_secs_f32())
        .map_err(reason)
}

fn reason(error: FromFileError) -> String {
    match error {
        FromFileError::IoError(error) => unreadable(error),
        error => error.to_string(),
    }
}
