// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the rules that a workspace applies, with the helpers that
//! build the shows they run on

mod editing;
mod history;
mod picture;
mod playback;
mod show;
mod video;

use super::{Workspace, playback::PANIC_FADE};
use crate::back::{
    Advance, Command, Cue, CueId, CueList, Edit, Kind, Media, Phase, Place, Problem, SILENCE,
};

/// Returns a workspace with one list of cues, every audio file of which
/// has loaded as ten seconds of sound
fn workspace(cues: Vec<Cue>) -> Workspace {
    let workspace = Workspace::new(vec![CueList::new("Main", cues)]);
    let loads: Vec<Command> = workspace
        .unloaded()
        .map(|cue| loaded(cue.id.0, &cue.file, sound(10.0)))
        .collect();
    loads.into_iter().fold(workspace, Workspace::apply)
}

fn sound(seconds: f32) -> Media {
    Media::Loaded {
        duration: seconds,
        sound: Some(seconds),
    }
}

fn loaded(id: u32, file: &str, media: Media) -> Command {
    Command::Loaded {
        cue: CueId(id),
        file: file.to_owned(),
        media,
    }
}

fn add(list: usize, kind: Kind, at: Place) -> Command {
    Command::Add { list, kind, at }
}

fn moving(id: u32, by: isize) -> Command {
    Command::Move { cue: CueId(id), by }
}

fn ids(workspace: &Workspace) -> Vec<u32> {
    workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.id.0)
        .collect()
}

fn standby(workspace: &Workspace) -> Option<u32> {
    workspace.lists()[0].standby.map(|id| id.0)
}

fn audio(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Audio, "Sound")
        .file("sound.ogg")
        .build()
}

fn playing(file: &str) -> Cue {
    Cue {
        file: file.to_owned(),
        ..audio(1)
    }
}

fn continuing(id: u32) -> Cue {
    Cue {
        advance: Advance::Continue,
        ..audio(id)
    }
}

fn continuing_after(id: u32, post_wait: f32) -> Cue {
    Cue {
        post_wait: post_wait.into(),
        ..continuing(id)
    }
}

/// Returns a fade cue that fades cue 1 out over four seconds and stops it
fn fade_out(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Fade, "Fade")
        .target(CueId(1))
        .duration(4.0)
        .build()
}

fn level(workspace: &Workspace, id: u32) -> Option<f32> {
    workspace.active_of(CueId(id)).map(|active| active.level)
}

fn opacity(workspace: &Workspace, id: u32) -> f32 {
    workspace
        .active_of(CueId(id))
        .expect("the cue runs")
        .opacity()
}

const GO: Command = Command::Go { list: 0 };
