// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Builders of the cues that play, group, and chain, and of a list of groups

use super::fixtures::workspace;
use crate::back::{Advance, Cue, CueId, Kind, Workspace};

pub(crate) fn audio(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Audio, "Sound")
        .file("sound.ogg")
        .build()
}

pub(super) fn picture(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Picture, "Slide")
        .file("slide.png")
        .build()
}

pub(crate) fn video(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Video, "Movie")
        .file("movie.mp4")
        .build()
}

pub(in crate::back) fn continued(cue: Cue) -> Cue {
    Cue {
        advance: Advance::Continue,
        ..cue
    }
}

pub(super) fn continuing(id: u32) -> Cue {
    continued(audio(id))
}

pub(in crate::back) fn continuing_after(id: u32, post_wait: f32) -> Cue {
    Cue {
        post_wait: post_wait.into(),
        ..continuing(id)
    }
}

pub(crate) fn memo(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Memo, "Note").build()
}

pub(crate) fn group(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Group, "Group").build()
}

/// Returns the cue at `depth` in groups
pub(super) fn within(depth: u8, cue: Cue) -> Cue {
    Cue { depth, ..cue }
}

/// Returns an audio cue that loops from 2 to 6 seconds into its file of ten
/// seconds, so that it reaches the end of its loop first after 6 seconds
/// and goes round the loop every 4
pub(super) fn vamp(id: u32) -> Cue {
    Cue {
        looping: true,
        loop_start: 2.0.into(),
        loop_end: 6.0.into(),
        ..audio(id)
    }
}

/// Returns the cue that waits `seconds` before it starts
pub(in crate::back) fn pre_waiting(seconds: f32, cue: Cue) -> Cue {
    Cue {
        pre_wait: seconds.into(),
        ..cue
    }
}

/// Returns the cue that loops the whole of its clip
pub(in crate::back) fn looped(cue: Cue) -> Cue {
    Cue {
        looping: true,
        ..cue
    }
}

/// Returns the cue that starts the next cue as it starts
pub(super) fn following(cue: Cue) -> Cue {
    Cue {
        advance: Advance::Follow,
        ..cue
    }
}

/// Returns a list with a group that holds a sound and a group of its own,
/// which holds a sound, followed by a sound outside them
pub(super) fn nested() -> Workspace {
    workspace(vec![
        audio(1),
        group(2),
        within(1, audio(3)),
        within(1, group(4)),
        within(2, audio(5)),
        audio(6),
    ])
}
