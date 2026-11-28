// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Watching the files that cues have read, for one that has changed, gone,
//! or come back
//!
//! A thread of its own looks at the stamp of each file at an interval, off
//! the update loop, and wakes the interface only for a file that differs
//! from what the cue read.  A show whose files stay as they are costs the
//! interface nothing between changes, which a timer that fired every
//! interval would not, since each message rebuilds the view.

use std::{path::PathBuf, thread, time::Duration};

use iced::futures::{Stream, channel::mpsc};
use itertools::Itertools;

use crate::back::{Cue, Stamp, Workspace};

/// File that a cue has read, as the cue saw it
///
/// The list of these identifies a watch, so that a file read anew, which
/// changes the stamp that its cue holds, starts the next look.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Watch {
    path: PathBuf,
    seen: Option<Stamp>,
}

impl Watch {
    pub fn all<'a>(workspace: &Workspace, cues: impl IntoIterator<Item = &'a Cue>) -> Vec<Self> {
        cues.into_iter()
            .filter_map(|cue| {
                Some(Self {
                    path: workspace.path(cue)?,
                    seen: cue.seen,
                })
            })
            .unique()
            .collect()
    }

    /// Returns whether the file is not as the cue saw it.  A cue that has
    /// no stamp holds a file that was absent or could not be opened, as a
    /// failure keeps no stamp for it, so that only opening the file tells
    /// whether it has changed, as a change of permission leaves the size
    /// and the time as they were.  Every other file costs one look at its
    /// stamp.
    fn changed(&self) -> bool {
        let now = Stamp::of(&self.path);
        match self.seen {
            Some(_) => now != self.seen,
            None => now.is_some_and(|stamp| stamp.when_readable(&self.path).is_some()),
        }
    }
}

/// Returns a stream that yields once for each `every` while a file of
/// `files` is not as its cue saw it, until the stream is dropped.  A file
/// that the cue read again ends the difference, since its next watch holds
/// the new stamp.
pub fn watch(files: Vec<Watch>, every: Duration) -> impl Stream<Item = ()> {
    let (sender, receiver) = mpsc::unbounded();
    thread::spawn(move || {
        while !sender.is_closed() {
            if files.iter().any(Watch::changed) && sender.unbounded_send(()).is_err() {
                return;
            }
            thread::sleep(every);
        }
    });
    receiver
}
