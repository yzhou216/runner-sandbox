// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Builders of the shows, cues, commands, and readings that the tests of a
//! workspace share, which the tests of the sound and video outputs borrow

use std::path::Path;

use super::Workspace;
use crate::back::{Active, Command, Cue, CueId, CueList, Edit, Kind, Media, Phase, Place, Problem};

/// Returns a workspace with one list of cues, whose picture files have
/// loaded as pictures and every other file as ten seconds of sound
pub(in crate::back) fn workspace(cues: Vec<Cue>) -> Workspace {
    lists_of(vec![cues])
}

/// Returns a workspace with one list of cues, none of whose files has loaded
pub(crate) fn fresh(cues: Vec<Cue>) -> Workspace {
    Workspace::new(vec![CueList::new("Main", cues)])
}

/// Returns a workspace with a list for each set of cues, loaded as for
/// [`workspace`]
pub(super) fn lists_of(lists: Vec<Vec<Cue>>) -> Workspace {
    let workspace = Workspace::new(
        lists
            .into_iter()
            .enumerate()
            .map(|(index, cues)| match index {
                0 => CueList::new("Main", cues),
                _ => CueList::new(format!("List {}", index + 1), cues),
            })
            .collect(),
    );
    let loads: Vec<Command> = workspace
        .unloaded()
        .map(|cue| {
            let media = match cue.kind {
                Kind::Picture => silent(0.0),
                _ => sound(10.0),
            };
            loaded(cue.id.0, &cue.file, media)
        })
        .collect();
    loads.into_iter().fold(workspace, Workspace::apply)
}

pub(crate) fn sound(seconds: f32) -> Media {
    Media::Loaded {
        duration: seconds,
        sound: Some(seconds),
        ahead: 0.0,
    }
}

/// Returns the media of a file that plays `seconds` of picture alone
pub(crate) fn silent(seconds: f32) -> Media {
    Media::Loaded {
        duration: seconds,
        sound: None,
        ahead: 0.0,
    }
}

pub(crate) fn loaded(id: u32, file: &str, media: Media) -> Command {
    Command::Loaded {
        cue: CueId(id),
        file: file.to_owned(),
        media,
        seen: None,
    }
}

/// Returns the command that records that the sound file of cue `id` is
/// not found
pub(super) fn not_found(id: u32) -> Command {
    loaded(id, "sound.ogg", Media::Failed("file not found".to_owned()))
}

pub(super) fn add(list: usize, kind: Kind, at: Place) -> Command {
    Command::Add {
        list,
        kind,
        at,
        count: 1,
    }
}

/// Returns the command that adds `count` cues in a row to the first list
pub(super) fn add_row(kind: Kind, at: Place, count: usize) -> Command {
    Command::Add {
        list: 0,
        kind,
        at,
        count,
    }
}

pub(super) fn paste(cues: Vec<Cue>, at: Place) -> Command {
    Command::Paste { list: 0, cues, at }
}

pub(super) fn cue_ids(ids: &[u32]) -> Vec<CueId> {
    ids.iter().map(|&id| CueId(id)).collect()
}

/// Returns the target of cue `id`, which is none for a cue that is not there
pub(super) fn target_of(workspace: &Workspace, id: u32) -> Option<CueId> {
    workspace.cue(CueId(id)).and_then(|cue| cue.target)
}

pub(super) fn collapse(ids: &[u32], collapsed: bool) -> Command {
    Command::Collapse {
        cues: cue_ids(ids),
        collapsed,
    }
}

pub(super) fn moving(id: u32, by: isize) -> Command {
    Command::Move { cue: CueId(id), by }
}

pub(super) fn edit(id: u32, edit: Edit) -> Command {
    Command::Edit(CueId(id), edit)
}

pub(super) fn rename(id: u32, name: &str) -> Command {
    edit(id, Edit::Name(name.to_owned()))
}

pub(super) fn delete(ids: &[u32]) -> Command {
    Command::Delete(cue_ids(ids))
}

pub(super) fn move_standby_in(list: usize, id: u32) -> Command {
    Command::Standby {
        list,
        cue: Some(CueId(id)),
    }
}

pub(super) fn move_standby(id: u32) -> Command {
    move_standby_in(0, id)
}

pub(super) fn ids(workspace: &Workspace) -> Vec<u32> {
    workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.id.0)
        .collect()
}

/// Returns the depths that the rows of the first list show, which a row
/// deeper than the rows above it allow does not keep
pub(super) fn depths(workspace: &Workspace) -> Vec<u8> {
    workspace.lists()[0].depths()
}

/// Returns the depth that each cue of the first list holds, whether the
/// rows above it allow it or not
pub(super) fn raw_depths(workspace: &Workspace) -> Vec<u8> {
    workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.depth)
        .collect()
}

/// Asserts the identities and the depths of the rows of the first list
pub(super) fn assert_rows(workspace: &Workspace, rows: &[u32], shown: &[u8]) {
    assert_eq!(
        (ids(workspace), depths(workspace)),
        (rows.to_vec(), shown.to_vec())
    );
}

pub(super) fn standby(workspace: &Workspace) -> Option<u32> {
    workspace.lists()[0].standby.map(|id| id.0)
}

/// Returns each cue that cannot play, with its identity
pub(super) fn problems(workspace: &Workspace) -> Vec<(u32, Problem<'_>)> {
    workspace
        .problems()
        .map(|(cue, problem)| (cue.id.0, problem))
        .collect()
}

/// Returns each cue that may fire without the operator, with the seconds
/// until it does
pub(super) fn ahead(workspace: &Workspace) -> Vec<(u32, f32)> {
    workspace
        .ahead()
        .into_iter()
        .map(|(cue, delay)| (cue.id.0, delay))
        .collect()
}

/// Returns the cues that GO has fired and that have not finished, in the
/// order that GO fired them
pub(super) fn running(workspace: &Workspace) -> Vec<u32> {
    workspace
        .active()
        .iter()
        .map(|active| active.id.0)
        .collect()
}

/// Returns the standby of the list `list`
pub(super) fn standby_of(workspace: &Workspace, list: usize) -> Option<u32> {
    workspace.lists()[list].standby.map(|id| id.0)
}

pub(super) fn phase_of(workspace: &Workspace, id: u32) -> Option<Phase> {
    workspace.active_of(CueId(id)).map(Active::phase)
}

pub(super) fn assert_phase(workspace: &Workspace, id: u32, phase: Phase) {
    assert_eq!(phase_of(workspace, id), Some(phase), "cue {id}");
}

pub(super) fn level(workspace: &Workspace, id: u32) -> Option<f32> {
    workspace.active_of(CueId(id)).map(|active| active.level)
}

pub(super) fn opacity(workspace: &Workspace, id: u32) -> f32 {
    workspace
        .active_of(CueId(id))
        .expect("the cue runs")
        .opacity()
}

pub(in crate::back) const GO: Command = Command::Go { list: 0 };

pub(super) fn shown(workspace: &Workspace) -> String {
    workspace.show(Some(Path::new("/show")))
}

/// Reads the text of a show in `/show`, which must write back as it was
pub(super) fn read_back(text: &str) -> Workspace {
    let read = Workspace::from_show(text, "/show").expect("the show reads back");
    assert_eq!(shown(&read), text);
    read
}
