// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Loading the files that audio, video, and picture cues name
//!
//! [`load_files`] finds how long each file plays, or why it cannot be
//! played, without decoding it.  Reading can block on a slow drive, so the
//! interface runs it off its own thread.

#[cfg(test)]
mod tests;
mod watch;

use std::{collections::HashMap, panic, path::PathBuf};

use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub use self::watch::{Watch, watch};
use super::{Command, Cue, CueId, Kind, Media, Stamp, Workspace, audio, video};

pub struct Load {
    cue: CueId,
    file: String,
    path: PathBuf,
    kind: Kind,
    /// Stamp that the file had when the cue last read it
    seen: Option<Stamp>,
}

impl Load {
    pub fn all<'a>(workspace: &Workspace, cues: impl IntoIterator<Item = &'a Cue>) -> Vec<Self> {
        cues.into_iter()
            .filter_map(|cue| {
                Some(Self {
                    cue: cue.id,
                    file: cue.file.clone(),
                    path: workspace.path(cue)?,
                    kind: cue.kind,
                    seen: cue.seen,
                })
            })
            .collect()
    }

    /// Returns whether the file has to be read, given the stamp that it has
    /// now.  A file that has changed or gone is read again, while one that
    /// stayed as it was keeps what reading it found, a failure that playing
    /// found included.
    fn stale(&self, now: Option<Stamp>) -> bool {
        now.is_none() || self.seen != now
    }
}

/// Reads the files of `loads` that no cue has read, or that have changed or
/// gone since, and returns the commands that record what reading found.
/// A show whose files stay as they are costs one look at the stamp of each.
/// Each distinct file is read once for each kind of cue that plays it, in
/// parallel, which shortens opening a large show whose files are not
/// cached yet.
pub fn load_files(loads: Vec<Load>) -> Vec<Command> {
    let paths: Vec<&PathBuf> = loads.iter().map(|load| &load.path).unique().collect();
    let stamps: HashMap<PathBuf, Option<Stamp>> = paths
        .par_iter()
        .map(|path| ((*path).clone(), Stamp::of(path)))
        .collect();
    let stale: Vec<Load> = loads
        .into_iter()
        .filter(|load| load.stale(stamps[&load.path]))
        .collect();
    // Collected first, so that rayon reads no file or a single one on this
    // thread instead of waking its pool
    let files: Vec<(PathBuf, Kind)> = stale
        .iter()
        .map(|load| (load.path.clone(), load.kind))
        .unique()
        .collect();
    let found: HashMap<&(PathBuf, Kind), Media> =
        files.par_iter().map(|file| (file, probe(file))).collect();
    stale
        .into_iter()
        .map(|load| {
            let media = found[&(load.path.clone(), load.kind)].clone();
            let seen = stamps[&load.path].and_then(|stamp| match media {
                Media::Failed(_) => stamp.when_readable(&load.path),
                _ => Some(stamp),
            });
            Command::Loaded {
                media,
                seen,
                cue: load.cue,
                file: load.file,
            }
        })
        .collect()
}

/// Reads how long a file plays for the kind of cue that plays it, and how
/// long its sound plays, or why it cannot be played.  A reader that panics
/// counts as one that cannot read the file, since the panic would end the
/// load of every file of the show, and the interface would wait for it for
/// good.
fn probe(file: &(PathBuf, Kind)) -> Media {
    panic::catch_unwind(|| read(file))
        .unwrap_or_else(|_| Media::Failed("cannot be read".to_owned()))
}

/// Reads the file for [`probe`].  The sound track of a video file has to
/// play as well, since the cue plays it along with the picture, while a
/// video file without one plays its picture alone.
fn read((path, kind): &(PathBuf, Kind)) -> Media {
    let loaded = match kind {
        Kind::Video => video::probe(path).and_then(|(duration, sound)| {
            let sound = sound
                .then(|| audio::probe(path))
                .transpose()
                .map_err(|reason| video_sound_reason(&reason))?;
            Ok((duration, sound))
        }),
        Kind::Picture => video::probe_still(path).map(|()| (0.0, None)),
        _ => audio::probe(path).map(|probed| (probed.duration, Some(probed))),
    };
    match loaded {
        Ok((duration, sound)) => Media::Loaded {
            duration,
            sound: sound.map(|probed| probed.duration),
            ahead: sound.map_or(0.0, |probed| probed.ahead),
        },
        Err(reason) => Media::Failed(reason),
    }
}

/// Tells why the sound track of a video file cannot play.  A video that
/// opened is no file of the wrong kind, so the reason of a file that no
/// decoder of sound reads gets words that blame the track.
fn video_sound_reason(reason: &str) -> String {
    match reason {
        audio::NOT_SOUND => "its sound is in a format that cinq cannot play".to_owned(),
        _ => format!("its sound cannot play: {reason}"),
    }
}
