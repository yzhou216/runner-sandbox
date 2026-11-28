// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Why cues cannot play, and the files of cues that have yet to load or
//! that could not be played

use crate::back::{Cue, Destination, Kind, Media, Problem, Workspace, osc};

impl Workspace {
    pub fn problem<'a>(&self, cue: &'a Cue) -> Option<Problem<'a>> {
        match cue.kind {
            Kind::Audio | Kind::Video | Kind::Picture => cue.file_problem(),
            Kind::Fade | Kind::Start | Kind::Stop | Kind::Pause | Kind::Devamp | Kind::GoTo => {
                match self.target(cue) {
                    Some(target) if !cue.can_target(target) => {
                        Some(if cue.kind == Kind::Devamp && target.id != cue.id {
                            Problem::NotAudioOrVideo
                        } else {
                            Problem::WrongTarget
                        })
                    }
                    Some(target) if cue.kind == Kind::Start && self.within(cue, target) => {
                        Some(Problem::InTarget)
                    }
                    Some(target) if cue.kind == Kind::Devamp && !target.loops() => {
                        Some(Problem::NotLooping)
                    }
                    Some(_) => None,
                    None => Some(Problem::NoTarget),
                }
            }
            Kind::Network if Destination::parse(&cue.destination).is_none() => {
                Some(if Destination::port_is_bad(&cue.destination) {
                    Problem::BadPort
                } else {
                    Problem::NoDestination
                })
            }
            Kind::Network if osc::packet(&cue.message).is_none() => {
                Some(if osc::has_open_quote(&cue.message) {
                    Problem::OpenQuote
                } else {
                    Problem::NoMessage
                })
            }
            Kind::Light | Kind::Network | Kind::Group | Kind::Wait | Kind::Memo => None,
        }
    }

    /// Returns whether `cue` lies in `target`, a group, at any depth.  A
    /// start cue that does cannot play, since the group would fire it again.
    fn within(&self, cue: &Cue, target: &Cue) -> bool {
        target.kind == Kind::Group
            && self
                .list_holding(cue.id)
                .is_some_and(|list| list.groups().holds(target.id, cue.id))
    }

    pub fn problems(&self) -> impl Iterator<Item = (&Cue, Problem<'_>)> {
        self.cues()
            .filter_map(|cue| self.problem(cue).map(|problem| (cue, problem)))
    }

    pub fn unloaded(&self) -> impl Iterator<Item = &Cue> {
        self.files_where(|media| *media == Media::Unloaded)
    }

    /// Returns the audio, video, and picture cues whose file has been read
    /// and whose cue is not running, to look at again, since the file may
    /// have changed, or gone, or come back.  A running cue keeps playing
    /// from the file that it opened, and the next look finds what became of
    /// it.
    pub fn to_recheck(&self) -> impl Iterator<Item = &Cue> {
        self.files_where(|media| *media != Media::Unloaded)
            .filter(|cue| self.active_of(cue.id).is_none())
    }

    fn files_where(&self, test: fn(&Media) -> bool) -> impl Iterator<Item = &Cue> {
        self.cues()
            .filter(move |cue| cue.kind.has_file() && test(&cue.media) && cue.file_name().is_some())
    }
}
