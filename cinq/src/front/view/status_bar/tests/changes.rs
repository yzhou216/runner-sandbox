// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what the notice of a change tells while the outputs show, and how
//! it shortens
mod fitting;
mod silence;
mod telling;
mod warnings;

use unicode_width::UnicodeWidthStr;

use super::{
    super::{hints::notices::notice, hints_in, name},
    cancel::COLUMNS,
    opened,
};
use crate::{
    back::{Command, Cue, CueId, Edit, Kind, OutputEdit, OutputId, Sends, Workspace, fresh},
    front::{App, app::Notices, update::Clipped, view::format},
};

/// Returns a show of one cue that plays a file, with a second output
fn rain() -> Workspace {
    let cue = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .file("rain.ogg")
        .build();
    fresh(vec![cue]).apply(Command::add_output(1, 1))
}

fn rename(id: u32, name: &str) -> Command {
    Command::EditOutput(OutputId(id), OutputEdit::Name(name.to_owned()))
}

/// Returns the hints beside the status of the app that shows `workspace`,
/// which `room` columns hold
fn told(workspace: Workspace, room: usize) -> String {
    let app = App {
        workspace,
        ..App::default()
    };
    name::shown(&app, room).1
}
