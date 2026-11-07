// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of opening and saving shows, and of keeping their unsaved changes
//! as long as they last

mod keeper;
mod opening;
mod saving;

use std::fs;

use tempfile::TempDir;

use super::{file::write, *};
use crate::back::{Command, CueId, Edit, Kind, Place};

/// Show file of one list of one memo cue
const SHOW: &str = "[[list]]\nname = \"Main\"\n\n[[list.cue]]\nid = 1\nnumber = \"1\"\nkind = \"memo\"\nname = \"Note\"\n";

/// File of the storm show, in `/shows`
const STORM: &str = "/shows/storm.cinq.toml";

/// File of the calm show, in `/shows`
const CALM: &str = "/shows/calm.cinq.toml";

/// Returns what reading the show in `file` found: `text` in its file, and
/// `kept` among the unsaved changes
fn found(file: Option<&str>, text: Option<&str>, kept: Option<&str>) -> Read {
    Read {
        file: file.map(PathBuf::from),
        directory: PathBuf::from("/shows"),
        text: Ok(text.map(str::to_owned)),
        kept: Ok(kept.map(str::to_owned)),
    }
}

/// Returns the state of the interface with the storm show open
fn storm() -> App {
    App::default().opened(found(Some(STORM), Some(SHOW), None))
}

/// Renames the memo cue of the storm show
fn renamed(app: App, name: &str) -> App {
    app.send(Command::Edit(CueId(1), Edit::Name(name.to_owned())))
}

/// Returns what saving the show of `app` in `file` finds when it succeeds
fn saving(app: &App, file: &str) -> Saved {
    let file = PathBuf::from(file);
    Saved {
        show: app.show.id,
        text: app.workspace.show(file.parent()),
        file,
        result: Ok(()),
    }
}

/// Returns a keeper of unsaved changes in a new temporary directory, along
/// with the directory, which goes once the test drops it
fn keeper() -> (Keeper, TempDir) {
    let directory = TempDir::new().expect("a temporary directory can be made");
    (Keeper::new(Some(directory.path().to_owned())), directory)
}
