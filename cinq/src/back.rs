// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Show model, with audio, video, light, and network playback, with no
//! knowledge of any interface
//!
//! A front end reads a [`Workspace`] and changes it only by applying a
//! [`Command`], so every rule about what may change lives here.  Every
//! change consumes the workspace and returns a new one.  [`Audio`],
//! [`Video`], [`Lights`], [`Network`], [`osc_listen`], and [`load_files`]
//! are the parts with side effects: they make the sound and pictures,
//! send the light levels and the messages of network cues, receive OSC
//! commands, and read the files that cues name.

mod audio;
mod cue;
mod failure;
mod items;
mod light;
mod load;
mod network;
mod osc;
mod output;
mod patch;
mod video;
mod workspace;

use std::time::Duration;

#[cfg(test)]
use cue::Loop;
use cue::{Clip, Pass, Shown};

pub use audio::{Audio, Fault, READY_AHEAD, SOUND_FILES, devices as audio_devices};
pub use cue::{
    Advance, Cue, CueId, CueList, Edit, Groups, Kind, LEVELS, LOUDEST, Media, PERCENT, Problem,
    SILENCE, Stamp, one_line, parse_level, parse_percent, parse_send,
};
pub use light::{Lights, Reports as LightReports};
pub use load::{Load, Watch, load_files, watch};
pub use network::Network;
pub use osc::{Destination, Heard, Outgoing, Remote, listen as osc_listen, packet as osc_packet};
pub use output::{
    Channels, DEVICE_CHANNELS, Device, Output, OutputEdit, OutputId, Sends, output_moved_between,
};
pub use patch::{Instrument, InstrumentId, PatchEdit, parse_address};
pub use video::{PICTURE_FILES, Picture, VIDEO_FILES, Video};
pub use workspace::{Active, Change, Command, Phase, Place, Turn, Unlit, Workspace};

/// Cues and workspaces that the tests of the front end build, and the
/// readings that they take of them, as those of the back end do
#[cfg(test)]
pub(crate) use workspace::tests::{
    audio, fresh, group, ids, loaded, memo, silent, sound, targets, video,
};

/// Interval of the playback clock while any cue is running, which the front
/// end keeps and the sound output glides by
pub const TICK: Duration = Duration::from_millis(25);
