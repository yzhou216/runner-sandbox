// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What loading the file of a cue found, and why a cue cannot play

use std::{fs, path::Path, time::SystemTime};

use strum::Display;

use super::Cue;

/// Size and modification time of a file, by which a later look tells that it
/// has changed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stamp {
    len: u64,
    modified: Option<SystemTime>,
}

impl Stamp {
    /// Returns the stamp of the file at `path`, or none where it is not
    /// there.  This reads no more than the directory entry, so that
    /// looking at every file of a show costs little.
    pub fn of(path: &Path) -> Option<Self> {
        let found = fs::metadata(path).ok()?;
        Some(Self {
            len: found.len(),
            modified: found.modified().ok(),
        })
    }

    /// Returns the stamp as a failure of the file at `path` keeps it, which
    /// is none where the file cannot be opened.  Such a file may become
    /// readable with no change to its size or time, as a change of
    /// permission makes it, so that the failure must be looked at again,
    /// while one that opens and fails by its contents stays until the file
    /// changes.
    pub fn when_readable(self, path: &Path) -> Option<Self> {
        fs::File::open(path).ok().map(|_| self)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Media {
    #[default]
    Unloaded,
    /// File plays for `duration` seconds, and its sound for `sound`
    /// seconds, apart from a video file with a picture alone, which holds
    /// no sound.  The two are the same for a sound file, while the sound
    /// of a video file can end before its picture or after it.  A picture
    /// file, which shows until something stops its cue, plays for none and
    /// holds no sound.  The sound is decoded `ahead` seconds ahead of where
    /// it plays, which is none for a file without one.
    Loaded {
        duration: f32,
        sound: Option<f32>,
        ahead: f32,
    },
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Problem<'a> {
    #[strum(to_string = "no file set")]
    NoFile,
    #[strum(to_string = "file not loaded")]
    NotLoaded,
    #[strum(transparent)]
    LoadFailed(&'a str),
    #[strum(to_string = "starts at or after its end")]
    EmptyClip,
    #[strum(to_string = "starts past the end of its file")]
    PastFile,
    /// Loop that starts at or after its end, once both lie within the clip
    #[strum(to_string = "loop starts at or after its end")]
    EmptyLoop,
    /// Loop that lasts less than a millisecond, which kira cannot play
    #[strum(to_string = "loop is shorter than a millisecond")]
    ShortLoop,
    #[strum(to_string = "no target cue")]
    NoTarget,
    /// Control cue names a cue that it cannot act on, such as itself, or a
    /// group for a fade or a devamp
    #[strum(to_string = "cannot act on its target cue")]
    WrongTarget,
    /// Devamp cue targets a cue that is no audio or video cue, such as a
    /// group, which has no loop to leave
    #[strum(to_string = "target is not audio or video")]
    NotAudioOrVideo,
    /// Start cue lies in the group that it starts, which would fire it
    /// again and so never end
    #[strum(to_string = "lies in the group that it starts")]
    InTarget,
    #[strum(to_string = "target cue does not loop")]
    NotLooping,
    #[strum(to_string = "no destination as host:port")]
    NoDestination,
    #[strum(to_string = "port is not a number from 1 to 65535")]
    BadPort,
    #[strum(to_string = "no OSC message as /address arguments")]
    NoMessage,
    #[strum(to_string = "OSC message leaves a double quote open")]
    OpenQuote,
}

impl Cue {
    /// Returns why the file of an audio, video, or picture cue cannot play,
    /// which no other kind of cue has
    pub fn file_problem(&self) -> Option<Problem<'_>> {
        match &self.media {
            _ if self.file_name().is_none() => Some(Problem::NoFile),
            Media::Unloaded => Some(Problem::NotLoaded),
            Media::Failed(reason) => Some(Problem::LoadFailed(reason)),
            Media::Loaded { .. } if self.loops_nothing() => Some(Problem::EmptyLoop),
            Media::Loaded { .. } if self.loops_briefly() => Some(Problem::ShortLoop),
            Media::Loaded { duration, .. } if self.kind.plays_file() && self.clip().is_none() => {
                Some(if self.start.value() >= *duration {
                    Problem::PastFile
                } else {
                    Problem::EmptyClip
                })
            }
            Media::Loaded { .. } => None,
        }
    }
}
