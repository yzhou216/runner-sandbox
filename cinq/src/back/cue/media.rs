// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What loading the file of a cue found, and why a cue cannot play

use strum::Display;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Media {
    /// File has not been loaded since it was set
    #[default]
    Unloaded,
    /// File plays for `duration` seconds, and its sound for `sound`
    /// seconds, apart from a video file with a picture alone, which holds
    /// no sound.  The two are the same for a sound file, while the sound
    /// of a video file can end before its picture or after it.  A picture
    /// file, which shows until something stops its cue, plays for none and
    /// holds no sound.
    Loaded { duration: f32, sound: Option<f32> },
    /// File cannot be played, for the given reason
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
    /// Audio or video cue starts at or after its end, or past the end of
    /// its file, so it has nothing to play
    #[strum(to_string = "starts at or after its end")]
    EmptyClip,
    #[strum(to_string = "no target cue")]
    NoTarget,
}
