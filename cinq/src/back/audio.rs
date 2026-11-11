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
//! skip.  The sound plays on the default audio device, or on the one that
//! the configuration file names.

mod device;
mod preload;
#[cfg(test)]
mod tests;
mod voice;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend,
    backend::cpal::CpalBackendSettings,
    sound::{FromFileError, streaming::StreamingSoundData},
    track::MainTrackBuilder,
};

use super::{
    Active, Clip, Command, Cue, CueId, Problem, Workspace,
    load::{unplayable, unreadable},
};
pub use device::devices;
use preload::{Ask, Preloader, Wanted};
use voice::Voice;

/// Endings of the names of the sound files that the sound output plays,
/// which the features of kira in `Cargo.toml` decide
pub const SOUND_FILES: [&str; 5] = ["flac", "m4a", "mp3", "ogg", "wav"];

/// Sounds that can play at once.  kira refuses a sound past them, which
/// cinq takes for a broken file, so there are far more than kira's 128 and
/// than any show plays at once.
const SOUNDS: usize = 1024;

/// Seconds before a sound starts that it is made ready, unless the
/// configuration file sets another number
pub const READY_AHEAD: f64 = 10.0;

pub struct Audio {
    output: Result<AudioManager, String>,
    /// ID of the audio device that the output was asked to open, or none
    /// for the default device
    device: Option<String>,
    voices: HashMap<CueId, Voice>,
    preloader: Preloader,
}

impl Audio {
    /// Opens the audio device whose ID is `device`, or the default audio
    /// device without one.  Without a device to open, the workspace still
    /// runs and files still load, but nothing is heard.
    pub fn open(device: Option<&str>) -> Self {
        Self {
            output: output(device),
            device: device.map(str::to_owned),
            voices: HashMap::new(),
            preloader: Preloader::new(),
        }
    }

    /// Opens the audio device whose ID is `device`, or the default one
    /// without it, unless the sound plays on it already.  An output that
    /// failed to open tries again, so that a device plugged in since can
    /// be reached.  The sounds that play end with the old output, and the
    /// next sync starts them on the new one from where they are.  The sounds
    /// made ready ahead are made again, or a cue that plays and that the
    /// next GO fires again would start over from the sound made ready for
    /// that GO.
    pub fn route(&mut self, device: Option<&str>) {
        if self.output.is_err() || self.device.as_deref() != device {
            *self = Self::open(device);
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.output.as_ref().err().map(String::as_str)
    }

    /// Makes the sound match the workspace, and returns the workspace with
    /// the files whose sound could not start.  The sounds that runs may
    /// start soon are made ready once those that start now have taken
    /// theirs, unless there is no audio device to hear them on.
    pub fn sync(&mut self, workspace: Workspace, ahead: f64) -> Workspace {
        self.preloader.collect();
        let failures = self.play(&workspace);
        if self.output.is_ok() {
            self.preloader.want(upcoming(&workspace, ahead));
        }
        failures.into_iter().fold(workspace, Workspace::apply)
    }

    fn play(&mut self, workspace: &Workspace) -> Vec<Command> {
        let Ok(manager) = &mut self.output else {
            return Vec::new();
        };
        let preloader = &mut self.preloader;
        let sounding: HashMap<CueId, (&Cue, &Active, Clip)> = workspace
            .active()
            .iter()
            .filter_map(|active| {
                let cue = workspace.cue(active.id)?;
                Some((cue.id, (cue, active, active.sound()?)))
            })
            .collect();
        // Voices that play on, and those whose cue has stopped or which GO
        // has fired again
        let (mut kept, ended): (HashMap<_, _>, HashMap<_, _>) = std::mem::take(&mut self.voices)
            .into_iter()
            .partition(|(id, voice)| {
                sounding
                    .get(id)
                    .is_some_and(|(_, active, _)| active.run == voice.run)
            });
        ended.into_values().for_each(Voice::stop);
        let (voices, failures): (_, Vec<Command>) = sounding
            .into_iter()
            .map(|(id, (cue, active, clip))| {
                let voice = match kept.remove(&id) {
                    Some(voice) => Ok(voice),
                    None => started(manager, preloader, workspace.path(cue), active, clip),
                };
                voice
                    .map(|voice| (id, voice.followed(active)))
                    .map_err(|reason| unplayable(cue, reason))
            })
            .partition_result();
        self.voices = voices;
        let heard = self
            .voices
            .iter()
            .filter_map(|(&id, voice)| voice.heard(workspace.active_of(id)?));
        failures.into_iter().chain(heard).collect()
    }
}

fn output(device: Option<&str>) -> Result<AudioManager, String> {
    let found = device.map(device::find).transpose()?;
    AudioManager::<DefaultBackend>::new(AudioManagerSettings {
        backend_settings: CpalBackendSettings {
            device: found,
            ..CpalBackendSettings::default()
        },
        main_track_builder: MainTrackBuilder::new().sound_capacity(SOUNDS),
        ..AudioManagerSettings::default()
    })
    .map_err(|error| match device {
        Some(id) => format!("cannot play on {id}: {error}"),
        None => error.to_string(),
    })
}

fn started(
    manager: &mut AudioManager,
    preloader: &mut Preloader,
    path: Option<PathBuf>,
    active: &Active,
    clip: Clip,
) -> Result<Voice, String> {
    let path = path.ok_or_else(|| Problem::NoFile.to_string())?;
    let wanted = Wanted { path, clip };
    match preloader.take(active.id, &wanted) {
        Some(Ok(sound)) => Voice::start(manager, sound, active),
        // Sound that could not be made ready, whose file may have come back
        // since
        _ => Voice::open(manager, &wanted.path, active, clip),
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
