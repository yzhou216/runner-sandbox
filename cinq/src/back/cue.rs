// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue data model, with the cue lists that hold cues

mod clip;
mod edit;
mod kind;
mod levels;
mod list;
mod media;
mod seconds;
#[cfg(test)]
mod tests;

use std::{collections::BTreeMap, ffi::OsStr, fmt, path::Path};

use bon::Builder;

use super::{InstrumentId, output::Sends};

#[cfg(test)]
pub(super) use clip::Loop;
pub(super) use clip::{Clip, Pass, Shown};
pub use edit::Edit;
pub use kind::{Advance, Kind};
pub use levels::{LEVELS, LOUDEST, PERCENT, SILENCE, parse_level, parse_percent, parse_send};
pub use list::{CueList, Groups};
pub use media::{Media, Problem, Stamp};
pub use seconds::Seconds;

/// Identity of a cue that stays stable when the operator renumbers it
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CueId(pub u32);

/// One cue of a cue list, which displays as its number and name, or as its
/// name alone when it has no number
///
/// [`Cue::new`] takes the settings every cue needs and returns a builder
/// for the rest, which `build` finishes.  By default a cue has no target,
/// no waits, and a manual advance, an audio or video cue plays its whole
/// file once, and a picture cue shows its file until something stops it.
/// Light, fade, and wait cues last five seconds, light cues set no levels,
/// and fade cues fade their target out and stop it, which is what most
/// fades do.
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
    #[builder(skip)]
    pub media: Media,
    /// Stamp that the file had when `media` was found, which tells the next
    /// look at the file whether to read it again.  A file that was not
    /// there, or could not be opened, has none, and is read again at every
    /// look.
    #[builder(skip)]
    pub seen: Option<Stamp>,
    /// Time in the file at which an audio or video cue starts
    #[builder(default, into)]
    pub start: Seconds,
    /// Time in the file at which an audio or video cue ends, which is the
    /// end of the file when the text reads as no time
    #[builder(default, into)]
    pub end: Seconds,
    /// Whether an audio or video cue plays from the start of its loop again
    /// each time it reaches the end of its loop, until something stops it or
    /// a devamp cue lets it go on from its loop to the end of its clip
    #[builder(default)]
    pub looping: bool,
    /// Time in the file at which the loop starts, which is the start of the
    /// cue when the text reads as no time
    #[builder(default, into)]
    pub loop_start: Seconds,
    /// Time in the file at which the loop ends, which is the end of the cue
    /// when the text reads as no time
    #[builder(default, into)]
    pub loop_end: Seconds,
    /// How many groups the cue lies in, as far as the rows above it allow,
    /// as [`CueList::depths`] tells
    #[builder(default)]
    pub depth: u8,
    /// Whether a group cue hides the cues that it holds in the cue list.
    /// The show file keeps it, so that a show opens as it was left, but it
    /// is no change to undo, as a closed fold is none in Vim.
    #[builder(default)]
    pub collapsed: bool,
    /// Cue that a fade, start, stop, pause, devamp, or GoTo cue acts on
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
    /// Levels at which an audio or video cue sends to outputs, or none to
    /// send to the first output at 0 dB, as a cue of a show file that names
    /// no outputs does
    pub sends: Option<Sends>,
    /// Levels in percent that a light cue fades instruments to
    #[builder(default)]
    pub lights: BTreeMap<InstrumentId, f32>,
    /// Where a network cue sends its message, as [`crate::back::Destination`]
    /// reads it
    #[builder(default, into)]
    pub destination: String,
    /// OSC message that a network cue sends, as [`crate::back::osc`] reads
    /// it
    #[builder(default, into)]
    pub message: String,
}

/// Returns `text` on one line, with every run of white space, line breaks
/// and tabs included, as one space, so that a name or a note from a show
/// file never takes a second line of a display that has room for one
pub fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl fmt::Display for Cue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = one_line(&self.name);
        if self.number.trim().is_empty() {
            f.write_str(&name)
        } else {
            write!(f, "{} {name}", self.number)
        }
    }
}

impl Cue {
    fn new_name(kind: Kind) -> String {
        format!("New {kind} cue")
    }

    /// Returns the edit that makes the cue play or show `file`, which names
    /// the cue after the file as well while the cue goes by the name it was
    /// added with, or by the name of the file it had before, as QLab names
    /// the cues of files.  A path without a name for the cue, such as an
    /// emptied field, gives back the name the cue was added with, and the
    /// name follows the path again once a path is typed anew.
    pub fn playing(&self, file: String) -> Edit {
        let added = Self::new_name(self.kind);
        let unnamed = self.name == added || Some(self.name.as_str()) == stem(&self.file);
        match (stem(&file), unnamed) {
            (Some(name), true) => Edit::FileAndName {
                name: name.to_owned(),
                file,
            },
            (None, true) => Edit::FileAndName { name: added, file },
            (_, false) => Edit::File(file),
        }
    }

    /// Returns the file that the cue names, without the spaces around it,
    /// which do not count, so that a pasted path with a trailing newline
    /// still loads; or none when the cue names no file
    pub fn file_name(&self) -> Option<&str> {
        Some(self.file.trim()).filter(|name| !name.is_empty())
    }

    /// Returns the length of the cue's action, which for an audio or video
    /// cue is its clip from start to end, without going round its loop.  It
    /// is zero for instant cues, for audio and video cues without a clip to
    /// play, and for picture cues, whose action has no end.
    pub fn action(&self) -> f32 {
        match self.kind {
            kind if kind.plays_file() => self.clip().map_or(0.0, |clip| clip.length() as f32),
            kind if kind.has_duration() => self.duration.value(),
            _ => 0.0,
        }
    }

    /// Returns whether the action of the cue has no end, which a cue that
    /// cannot play never has, since GO skips it
    fn endless(&self) -> bool {
        (self.kind == Kind::Picture || self.loops()) && self.file_problem().is_none()
    }

    pub fn can_target(&self, other: &Cue) -> bool {
        other.id != self.id
            && match self.kind {
                Kind::Fade => other.kind.has_file(),
                Kind::Devamp => other.kind.plays_file(),
                Kind::Start | Kind::Stop | Kind::Pause | Kind::GoTo => true,
                Kind::Audio
                | Kind::Video
                | Kind::Picture
                | Kind::Light
                | Kind::Network
                | Kind::Group
                | Kind::Wait
                | Kind::Memo => false,
            }
    }

    /// Returns the delay between this cue firing and it firing the next cue
    /// of the list of `groups`, or `None` when the next cue waits for GO
    fn next_delay_in(&self, groups: &Groups) -> Option<f32> {
        match self.advance {
            Advance::Manual => None,
            Advance::Continue => Some(self.pre_wait.value() + self.post_wait.value()),
            Advance::Follow if groups.endless(self) => None,
            Advance::Follow => Some(self.pre_wait.value() + groups.action_of(self)),
        }
    }
}

fn stem(file: &str) -> Option<&str> {
    Path::new(file.trim()).file_stem().and_then(OsStr::to_str)
}
