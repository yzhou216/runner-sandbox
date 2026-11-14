// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Show model, with audio and video playback, with no knowledge of any
//! interface
//!
//! A front end reads a [`Workspace`] and changes it only by applying a
//! [`Command`], so every rule about what may change lives here.  Every
//! change consumes the workspace and returns a new one, without side
//! effects.  [`Audio`], which makes the sound that the workspace describes,
//! [`Video`], which makes its pictures, and [`load_files`], which reads the
//! files that cues name, are the parts with side effects.

mod audio;
mod cue;
mod items;
mod load;
mod output;
mod video;
mod workspace;

use std::time::Duration;

use cue::{Clip, Shown};

pub use audio::{Audio, READY_AHEAD, SOUND_FILES, devices as audio_devices};
pub use cue::{
    Advance, Cue, CueId, CueList, Edit, Kind, LEVELS, LOUDEST, Media, Problem, SILENCE, parse_level,
};
pub use load::{Load, load_files};
pub use output::{Channels, Device, Output, OutputEdit, OutputId, Sends};
pub use video::{PICTURE_FILES, Picture, VIDEO_FILES, Video};
pub use workspace::{Active, Change, Command, Phase, Place, Turn, Workspace};

/// Cues and workspaces that the tests of the front end build as those of
/// the back end do
#[cfg(test)]
pub(crate) use workspace::tests::{audio, loaded, silent, sound};

/// Interval of the playback clock while any cue is running, which the front
/// end keeps and the sound output glides by
pub const TICK: Duration = Duration::from_millis(25);
