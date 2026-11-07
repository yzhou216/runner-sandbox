// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue data model, with the cue lists that hold cues

mod clip;
mod edit;
mod kind;
mod list;
mod media;
mod seconds;
#[cfg(test)]
mod tests;

use std::{ffi::OsStr, fmt, ops::RangeInclusive, path::Path};

use bon::Builder;

pub use clip::{Clip, Shown, Wanted};
pub use edit::Edit;
pub use kind::{Advance, Kind};
pub use list::CueList;
pub use media::{Media, Problem};
pub use seconds::Seconds;

/// Level in dB that counts as silence, which is the bottom of every level
pub const SILENCE: f32 = -60.0;

/// Loudest level in dB that the operator can give a cue
pub const LOUDEST: f32 = 12.0;

/// Levels in dB that the operator can give a cue
pub const LEVELS: RangeInclusive<f32> = SILENCE..=LOUDEST;

/// Returns the level in dB that `text` reads as, with its unit or without
/// it, as in `-6` or `-6 dB`, if it lies between [`SILENCE`] and
/// [`LOUDEST`]
pub fn parse_level(text: &str) -> Option<f32> {
    text.trim()
        .trim_end_matches(['d', 'D', 'b', 'B'])
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|level| LEVELS.contains(level))
}

/// Identity of a cue that stays stable when the operator renumbers it
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CueId(pub u32);

/// One cue of a cue list, which displays as its number and name
///
/// [`Cue::new`] takes the settings every cue needs and returns a builder
/// for the rest, which `build` finishes.  By default a cue has no target,
/// no waits, and a manual advance, an audio or video cue plays its whole
/// file once, and a picture cue shows its file until something stops it.
/// Fade and wait cues last five seconds, and fade cues fade their target
/// out and stop it, which is what most fades do.
#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(start_fn = new)]
pub struct Cue {
    #[builder(start_fn)]
    pub id: CueId,
    #[builder(start_fn, into)]
    pub number: String,
    #[builder(start_fn)]
    pub kind: Kind,
    #[builder(start_fn, into)]
    pub name: String,
    /// File that an audio or video cue plays, or that a picture cue shows
    #[builder(default, into)]
    pub file: String,
    /// What loading the file found
    #[builder(skip)]
    pub media: Media,
    /// Time in the file at which an audio or video cue starts
    #[builder(default, into)]
    pub start: Seconds,
    /// Time in the file at which an audio or video cue ends, which is the
    /// end of the file when the text reads as no time
    #[builder(default, into)]
    pub end: Seconds,
    /// Whether an audio or video cue plays from its start again each time
    /// it reaches its end, until something stops it
    #[builder(default)]
    pub looping: bool,
    /// Cue a fade or stop cue acts on
    pub target: Option<CueId>,
    #[builder(default, into)]
    pub pre_wait: Seconds,
    /// Length of the action of a fade or wait cue
    #[builder(default = if kind.has_duration() { 5.0 } else { 0.0 }, into)]
    pub duration: Seconds,
    #[builder(default, into)]
    pub post_wait: Seconds,
    #[builder(default)]
    pub advance: Advance,
    #[builder(default, into)]
    pub notes: String,
    /// Level in dB an audio or video cue plays at, or a fade cue fades its
    /// target to
    #[builder(default = if kind == Kind::Fade { SILENCE } else { 0.0 })]
    pub level: f32,
    /// Whether a fade cue stops its target once the fade is over
    #[builder(default = kind == Kind::Fade)]
    pub stop_target: bool,
}

impl fmt::Display for Cue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.number, self.name)
    }
}

impl Cue {
    /// Returns the name that a new cue of `kind` goes by
    fn new_name(kind: Kind) -> String {
        format!("New {kind} cue")
    }

    /// Returns the edit that makes the cue play or show `file`, which names
    /// the cue after the file as well while the cue goes by the name it was
    /// added with, or by the name of the file it had before, as QLab names
    /// the cues of files
    pub fn playing(&self, file: String) -> Edit {
        let unnamed =
            self.name == Self::new_name(self.kind) || Some(self.name.as_str()) == stem(&self.file);
        match stem(&file) {
            Some(name) if unnamed => Edit::FileAndName {
                name: name.to_owned(),
                file,
            },
            _ => Edit::File(file),
        }
    }

    /// Returns the file that the cue names, without the spaces around it,
    /// which do not count, so that a pasted path with a trailing newline
    /// still loads; or none when the cue names no file
    pub fn file_name(&self) -> Option<&str> {
        Some(self.file.trim()).filter(|name| !name.is_empty())
    }

    /// Returns the length of the cue's action, which for an audio or video
    /// cue is one pass through its clip.  It is zero for instant cues, for
    /// audio and video cues without a clip to play, and for picture cues,
    /// whose action has no end.
    pub fn action(&self) -> f32 {
        match self.kind {
            kind if kind.plays_file() => self.clip().map_or(0.0, |clip| clip.length() as f32),
            kind if kind.has_duration() => self.duration.value(),
            _ => 0.0,
        }
    }

    /// Returns whether the action of the cue goes on until something stops
    /// it, as that of a picture cue does, and that of an audio or video cue
    /// that loops
    pub fn endless(&self) -> bool {
        self.kind == Kind::Picture || self.loops()
    }

    /// Returns whether this cue can act on `other`.  Fade cues act on audio,
    /// video, and picture cues, stop cues on any cue but themselves, and the
    /// remaining kinds on none.
    pub fn can_target(&self, other: &Cue) -> bool {
        other.id != self.id
            && match self.kind {
                Kind::Fade => other.kind.has_file(),
                Kind::Stop => true,
                Kind::Audio | Kind::Video | Kind::Picture | Kind::Wait | Kind::Memo => false,
            }
    }

    /// Returns the delay between this cue firing and it firing the next cue,
    /// or `None` when the next cue waits for GO, as it does after a cue that
    /// follows and whose action never ends, such as a picture cue
    fn next_delay(&self) -> Option<f32> {
        match self.advance {
            Advance::Manual => None,
            Advance::Continue => Some(self.pre_wait.value() + self.post_wait.value()),
            Advance::Follow if self.endless() => None,
            Advance::Follow => Some(self.pre_wait.value() + self.action()),
        }
    }
}

/// Returns the name of a file without its directory and its ending, or
/// none when there is no such name
fn stem(file: &str) -> Option<&str> {
    Path::new(file.trim()).file_stem().and_then(OsStr::to_str)
}
