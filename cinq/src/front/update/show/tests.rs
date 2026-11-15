// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of opening and saving shows, and of keeping their unsaved changes
//! as long as they last

mod keeper;
mod opening;
mod saving;

use std::{fs, sync::mpsc};

use tempfile::TempDir;

use super::{
    file::{kept_file, write},
    *,
};
use crate::back::{Command, CueId, Edit, Kind, Place};

/// Show file of one list of one memo cue
const SHOW: &str = "[[list]]\nname = \"Main\"\n\n[[list.cue]]\nid = 1\nnumber = \"1\"\nkind = \"memo\"\nname = \"Note\"\n";

const STORM: &str = "/shows/storm.cinq.toml";

const CALM: &str = "/shows/calm.cinq.toml";

fn found(file: Option<&str>, text: Option<&str>, kept: Option<&str>) -> Read {
    Read {
        file: file.map(PathBuf::from),
        directory: PathBuf::from("/shows"),
        text: Ok(text.map(str::to_owned)),
        kept: Ok(kept.map(str::to_owned)),
    }
}

fn storm() -> App {
    App::default().opened(found(Some(STORM), Some(SHOW), None))
}

fn renamed(app: App, name: &str) -> App {
    app.send(Command::Edit(CueId(1), Edit::Name(name.to_owned())))
}

fn saving(app: &App, file: &str) -> Saved {
    let file = PathBuf::from(file);
    Saved {
        show: app.show.id,
        text: app.workspace.show(file.parent()),
        file,
        result: Ok(()),
    }
}

/// Returns a new temporary directory, which goes once the test drops it
fn temporary() -> TempDir {
    TempDir::new().expect("a temporary directory can be made")
}

/// Returns a keeper of unsaved changes in a new temporary directory, along
/// with the directory
fn keeper() -> (Keeper, TempDir) {
    let directory = temporary();
    (Keeper::new(Some(directory.path().to_owned())), directory)
}

/// Holds the thread of `keeper` until the returned sender goes, as a slow
/// disk does, and returns once the thread is held
fn stalled(keeper: &Keeper) -> mpsc::Sender<()> {
    let (started, running) = mpsc::channel();
    let (release, held) = mpsc::channel::<()>();
    keeper.after(move || {
        let _ = started.send(());
        let _ = held.recv();
    });
    running.recv().expect("the thread runs what is sent");
    release
}
