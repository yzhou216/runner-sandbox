// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sound output, where the back end has side effects
//!
//! [`Audio::sync`] runs after every change to the workspace.  The workspace
//! stays the only record of what should be heard, so a sound that cannot
//! start, or that stops before its file ends, goes back to it as a file that
//! cannot be played.  A thread of its own makes ready the sounds that runs
//! may start next, so that GO waits on no file.  A thread for each audio
//! device opens it and tries again if it fails, so that a slow device never
//! stops the window.  The sound of a video file plays from the file's sound
//! track, along with the silence that AAC encoders put around the sound,
//! since the decoder reads no edit list, which marks that silence for
//! players to skip.  Each cue plays on the outputs that it sends to, which
//! play on channels of audio devices, as [`patch`] tells, and a
//! [`mixer`](sound::mixer) of cinq's own plays each device, since kira mixes
//! in stereo alone.

mod channels;
mod devices;
mod opening;
mod patch;
mod sound;
#[cfg(test)]
pub(super) mod tests;

use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, Instant},
};

use itertools::Itertools;
use kira::sound::{FromFileError, streaming::StreamingSoundData};

use super::{CueId, OutputId, Workspace, failure::unreadable};
pub use devices::{device::devices, fault::Fault};
use devices::{fault::RETRY, probes::Probes};
use patch::Patch;
use sound::{
    mixer::{Mixer, Mixers},
    opener::Opener,
    preload::{Ask, Preloader, Wanted},
    voice::Voice,
};

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
    patch: Patch,
    mixers: Mixers,
    /// Thread that opens the devices, which answers once each is open
    opener: Opener,
    voices: HashMap<CueId, Voice>,
    /// Latest run of each cue whose sound has started, which a sound that
    /// starts again for the same run is told from the first start by
    begun: HashMap<CueId, u32>,
    preloader: Preloader,
    /// When a device last answered an open, from which the devices that
    /// failed are tried again
    tried: Instant,
    /// When to look at the devices that have fewer channels than needed
    probes: Probes,
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
            opener: Opener::new(),
            voices: HashMap::new(),
            begun: HashMap::new(),
            preloader: Preloader::new(),
            tried: Instant::now(),
            probes: Probes::default(),
        }
    }

    /// Changes the device that outputs on the default device play on, as
    /// [`Audio::open`] takes it.  The next sync opens the new device and
    /// closes the old, which ends the sounds on the old one and starts them
    /// again on the new from where they are.  Outputs on named devices play
    /// on unbroken.
    pub fn route(&mut self, default: Option<&str>) {
        self.default = default.map(str::to_owned);
    }

    /// Returns why each output that cannot play cannot: its device is not
    /// there, cannot be opened, does not answer, or has stopped, or its
    /// channels lie past those of the device.  An output whose device is
    /// being opened for the first time tells nothing until the device
    /// answers, or for five seconds.
    pub fn problems(&self) -> Vec<(OutputId, Fault)> {
        self.patch
            .outputs()
            .iter()
            .filter_map(|patched| {
                if self.opener.unanswered(&patched.key) {
                    return Some((patched.id, Fault::Unanswered));
                }
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

    /// Returns how often the interface should look again while a device is
    /// being opened or asked about, or has failed in a way that is tried
    /// again, since only a sync takes in what the threads of the devices
    /// answer and tries a device again, and an idle interface runs none.
    /// A device that is fine, or that cannot be tried again, asks for no
    /// look.
    pub fn checks(&self) -> Option<Duration> {
        let failing = self.mixers.values().any(|mixer| match mixer {
            Ok(mixer) => mixer.failure().is_some(),
            Err(fault) => fault.retried(),
        });
        (failing || self.opener.busy()).then_some(RETRY)
    }

    /// Makes the sound match the workspace, and returns the workspace with
    /// the files whose sound could not start.  The sounds that runs may
    /// start soon are made ready once those that start now have taken
    /// theirs, unless there is no audio device to hear them on.
    pub fn sync(&mut self, workspace: Workspace, ahead: f64) -> Workspace {
        self.patched(Patch::of(workspace.outputs(), &self.default));
        self.arrived();
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
/// start soonest, then those that start cues that have yet to act will
/// fire, and those of the next GO of each list, through its start cues as
/// well.  Each sound is judged by its own start, so that sounds that start
/// together are never split, and those that start at the instant of GO are
/// always among them.
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
    let next = workspace.ahead().into_iter().filter_map(|(cue, delay)| {
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
    });
    fired.chain(next).unique_by(|ask| ask.cue).collect()
}

/// Frames that the streaming decoder of kira reads ahead of where its sound
/// plays, which is the size of its buffer of decoded frames
const READ_AHEAD: f64 = 16_384.0;

/// What a sound file tells without decoding its sound
#[derive(Debug, Clone, Copy)]
pub(super) struct Probed {
    /// Seconds that the sound plays
    pub duration: f32,
    /// Seconds of the sound that the decoder reads ahead of where it plays,
    /// which are more for a file whose rate is lower
    pub ahead: f32,
}

/// Reads how long the sound of a file plays, or why it cannot be played.
/// This opens the file and reads its format without decoding the sound.
pub(super) fn probe(file: &Path) -> Result<Probed, String> {
    let data = StreamingSoundData::from_file(file).map_err(reason)?;
    let channels = channels::of_wav(file).map_err(unreadable)?;
    if channels.is_some_and(|count| count > 2) {
        return Err("holds more than two channels".to_owned());
    }
    let duration = data.duration().as_secs_f64();
    Ok(Probed {
        duration: duration as f32,
        ahead: (READ_AHEAD * duration / data.num_frames().max(1) as f64) as f32,
    })
}

/// Reason of a file that no decoder of sound reads
pub(super) const NOT_SOUND: &str = "not a sound file that cinq can play";

/// Tells why a file cannot be played, in words of cinq's own where the
/// decoder fails, since its errors name the library and the stage, as
/// `unsupported feature: core (probe)` does
fn reason(error: FromFileError) -> String {
    match error {
        FromFileError::IoError(error) => unreadable(error),
        _ => NOT_SOUND.to_owned(),
    }
}
