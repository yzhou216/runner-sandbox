// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fields of the inspector, each a labelled input for one setting

use iced::{Element, Fill};
use strum::{IntoStaticStr, VariantArray};

use self::target::target_menu;
use super::locked::locked;
use crate::{
    back::{Advance, Cue, Destination, Edit, Kind, osc_packet},
    front::{
        App, Message, Setting,
        view::{dropdown, format},
    },
};

#[derive(Clone, Copy, PartialEq, IntoStaticStr)]
pub(super) enum Field {
    Number,
    Name,
    Target,
    Notes,
    #[strum(serialize = "Pre-wait")]
    PreWait,
    Duration,
    #[strum(serialize = "Post-wait")]
    PostWait,
    Then,
    Start,
    End,
    #[strum(serialize = "Loop start")]
    LoopStart,
    #[strum(serialize = "Loop end")]
    LoopEnd,
    #[strum(serialize = "Message")]
    Osc,
}

impl Field {
    pub(super) fn label(self, cue: &Cue) -> &'static str {
        match self {
            Field::Target if cue.kind.has_file() => "File",
            Field::Target if cue.kind == Kind::Network => "Destination",
            _ => self.into(),
        }
    }

    /// Returns whether the cue has a row for this field, which a kind with
    /// no file and no target has no use for
    pub(super) fn has_row(self, cue: &Cue) -> bool {
        !matches!(self, Field::Target)
            || !matches!(
                cue.kind,
                Kind::Light | Kind::Group | Kind::Wait | Kind::Memo
            )
    }

    /// Returns why the cue ignores the value of this field, such as the
    /// post-wait of an auto-follow cue, or the post-wait and Then setting of a
    /// cue in a group, which goes on to no cue
    fn ignored(self, cue: &Cue, inside: bool) -> Option<&'static str> {
        match self {
            Field::PostWait | Field::Then if inside => Some("in a group"),
            Field::PostWait if cue.advance == Advance::Follow => Some("with auto-follow"),
            Field::LoopStart | Field::LoopEnd if !cue.looping => Some("not looping"),
            _ => None,
        }
    }

    /// Returns the input for this field, or its value in a box that the
    /// keyboard skips where the show cannot be edited or the cue ignores it
    pub(super) fn input<'a>(self, app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
        let id = cue.id;
        let editable = app.editable();
        let inside = app.current().group_of(id).is_some();
        let ignored = self.ignored(cue, inside);
        let field =
            |value: &str, edit: fn(String) -> Edit| self.entry(app, cue, ignored, value, edit);
        match self {
            Field::Number => field(&cue.number, Edit::Number),
            Field::Name => field(&cue.name, Edit::Name),
            Field::Target => self.target(app, cue, ignored),
            Field::Notes => field(&cue.notes, Edit::Notes),
            Field::PreWait => field(cue.pre_wait.text(), |text| Edit::PreWait(text.into())),
            Field::Duration if cue.kind.has_duration() => {
                field(cue.duration.text(), |text| Edit::Duration(text.into()))
            }
            Field::Duration => shown(&format::action(&app.current().groups(), cue)),
            Field::PostWait => field(cue.post_wait.text(), |text| Edit::PostWait(text.into())),
            Field::Then if editable && !inside => dropdown(
                Advance::VARIANTS,
                Some(cue.advance),
                move |advance| Message::Edit(id, Edit::Advance(advance)),
                &app.config.keymap,
            )
            .into(),
            Field::Then => locked(&then_text(cue, inside), "", ignored, Fill),
            Field::Start => field(cue.start.text(), |text| Edit::Start(text.into())),
            Field::End => field(cue.end.text(), |text| Edit::End(text.into())),
            Field::LoopStart => field(cue.loop_start.text(), |text| Edit::LoopStart(text.into())),
            Field::LoopEnd => field(cue.loop_end.text(), |text| Edit::LoopEnd(text.into())),
            Field::Osc => field(&cue.message, Edit::Message),
        }
    }

    /// Returns the field that takes this one's place in the cue: a network
    /// cue has no duration, so its notes take that row, and its message,
    /// which is as long as a name, takes the row of the notes in the wide
    /// form
    pub(super) fn of(self, cue: &Cue) -> Self {
        match self {
            Field::Notes if cue.kind == Kind::Network => Field::Osc,
            Field::Duration if cue.kind == Kind::Network => Field::Notes,
            _ => self,
        }
    }

    fn unreadable(self, cue: &Cue) -> bool {
        match self {
            Field::PreWait => !cue.pre_wait.is_valid(),
            Field::Duration => cue.kind.has_duration() && !cue.duration.is_valid(),
            Field::PostWait => !cue.post_wait.is_valid(),
            Field::Start => !cue.start.is_valid(),
            Field::End => !cue.end.is_valid(),
            Field::LoopStart => !cue.loop_start.is_valid(),
            Field::LoopEnd => !cue.loop_end.is_valid(),
            _ => false,
        }
    }

    /// Returns whether the text of the field reads as a value, which only
    /// a time, a destination, or a message can fail to
    fn valid(self, cue: &Cue) -> bool {
        match self {
            Field::Target if cue.kind == Kind::Network => {
                Destination::parse(&cue.destination).is_some()
            }
            Field::Osc => osc_packet(&cue.message).is_some(),
            _ => !self.unreadable(cue),
        }
    }

    fn placeholder(self, cue: &Cue) -> &'static str {
        match self {
            Field::Target if cue.kind == Kind::Audio => "path of a sound file",
            Field::Target if cue.kind == Kind::Video => "path of a video file",
            Field::Target if cue.kind == Kind::Picture => "path of a picture file",
            Field::Target if cue.kind == Kind::Network => "host:port or udp://host:port",
            Field::Osc => "/address arguments",
            Field::PreWait | Field::Duration | Field::PostWait | Field::Start => "0",
            Field::End => "end of file",
            Field::LoopStart => "start of cue",
            Field::LoopEnd => "end of cue",
            _ => "",
        }
    }
}

/// Returns the sentence that tells which of the time fields in `columns`
/// reads as no time, if one does, since such a time counts as zero, except
/// for an end or a loop time, which is then left empty.  A field that the
/// cue ignores, `inside` a group or not, is passed over, since the view shows
/// it greyed out.
pub(super) fn unread(cue: &Cue, inside: bool, columns: &[&[Field]]) -> Option<String> {
    columns
        .iter()
        .copied()
        .flatten()
        .filter(|field| field.ignored(cue, inside).is_none())
        .find(|field| field.unreadable(cue))
        .map(|field| {
            let reading = match field {
                Field::End | Field::LoopStart | Field::LoopEnd => "reads as empty",
                _ => "counts as zero",
            };
            format!(
                "{} {reading}: use seconds or minutes:seconds",
                field.label(cue)
            )
        })
}

/// Returns the Then value that a locked box shows.  A cue in a group goes on
/// to no cue, so the box gives only the reason, which the narrow column
/// would otherwise cut after a long value such as `auto-continue`.
fn then_text(cue: &Cue, inside: bool) -> String {
    if inside {
        String::new()
    } else {
        cue.advance.to_string()
    }
}

/// Shows a value in a box that the keyboard skips, so it lines up with
/// editable ones
fn shown<'a>(value: &str) -> Element<'a, Message> {
    locked(value, "", None, Fill)
}

/// Returns the path that the File field of `cue` shows: what was typed into
/// it while the typing waits for the field to be left, or else the file
fn typed_file(app: &App, cue: &Cue) -> String {
    match &app.typed {
        Some((Setting::File(typed), text)) if *typed == cue.id => text.clone(),
        _ => cue.file.clone(),
    }
}

mod entry;
mod target;
#[cfg(test)]
mod tests;
