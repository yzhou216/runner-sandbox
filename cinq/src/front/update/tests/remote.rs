// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the commands that arrive over OSC, which do what their keys do

mod listening;
mod playing;
mod selecting;

use std::net::{IpAddr, Ipv4Addr};

use super::*;
use crate::{
    back::{Command, Remote},
    front::config::Config,
};

/// Returns the state of [`show`] once the files of its cues have loaded as
/// ten seconds of sound, so that its cues can play
fn playable(selected: Option<CueId>) -> App {
    let app = show(selected);
    let loads: Vec<Command> = app
        .workspace
        .cues()
        .map(|cue| loaded(cue.id.0, &cue.file, sound(10.0)))
        .collect();
    app.send_all(loads)
}

/// Host that the commands come from
const FROM: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5));

fn remote(app: App, remote: Remote) -> App {
    after(app, [Message::Remote(FROM, remote)])
}

fn running(app: &App) -> Vec<u32> {
    app.workspace.active().iter().map(|run| run.id.0).collect()
}
