// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeper of unsaved changes, and the thread of its own that writes and
//! reads the kept files of their shows

use std::{
    iter,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

use itertools::{Either, Itertools};

use super::{
    Show,
    file::{kept_file, remove, write},
    read,
};

/// The keeper decides at once what to write and what to remove, and leaves
/// the writing to a thread of its own, so that a slow disk never holds up
/// GO, Panic, or the clock.  The thread makes the changes in the order that
/// they came, makes only the newest change to a file while several wait,
/// and makes every change sent to it before the keeper goes, as the keeper
/// does when the window closes.  The same thread reads the kept files for
/// the [`Reader`]s, each after the changes that came before the asking.
pub struct Keeper {
    /// Directory that keeps unsaved changes, if the platform has one
    unsaved_directory: Option<PathBuf>,
    /// Identity and version of the show last seen
    seen: Option<(u64, u64)>,
    /// Kept file written last, with the identity of its show
    kept: Option<(u64, PathBuf)>,
    /// Way to the thread that makes the changes to the kept files, which
    /// closes when the keeper and its readers go, so that the thread ends
    jobs: mpsc::Sender<Job>,
    /// Why the changes that failed did, in the order that they failed
    failures: mpsc::Receiver<String>,
}

impl Keeper {
    pub fn new(unsaved_directory: Option<PathBuf>) -> Self {
        let (jobs, waiting) = mpsc::channel();
        let (failed, failures) = mpsc::channel();
        thread::spawn(move || work(&waiting, &failed));
        Self {
            unsaved_directory,
            seen: None,
            kept: None,
            jobs,
            failures,
        }
    }

    /// Keeps the unsaved changes of `show` in its kept file, or removes the
    /// file once the show has none, as well as the kept file that the show
    /// had before it moved to another file.  The kept file of another show
    /// stays, for that show to bring back.  Returns why keeping failed, if
    /// it did.  A change that fails on the thread is told at a later call,
    /// once the thread has tried it.
    pub fn keep(&mut self, show: &Show) -> Option<String> {
        let failed = self.failures.try_iter().last();
        if self.seen == Some((show.id, show.version)) {
            return failed;
        }
        self.seen = Some((show.id, show.version));
        let wanted = self
            .unsaved_directory
            .as_deref()
            .filter(|_| show.unsaved())
            .map(|directory| kept_file(directory, show.file.as_deref()));
        let gone = self
            .kept
            .take()
            .filter(|(id, kept)| *id == show.id && wanted.as_ref() != Some(kept))
            .map(|(_, kept)| kept);
        if let Some(gone) = gone {
            self.send(Change::Remove(gone));
        }
        if let Some(wanted) = &wanted {
            self.send(Change::Write(wanted.clone(), show.text.clone()));
        }
        self.kept = wanted.map(|wanted| (show.id, wanted));
        failed.or_else(|| {
            (self.unsaved_directory.is_none() && show.unsaved())
                .then(|| "Cannot keep the unsaved changes: there is no state directory".to_owned())
        })
    }

    /// Returns the way to read the kept files, or none when the platform
    /// has no directory to keep them in
    pub fn reader(&self) -> Option<Reader> {
        self.unsaved_directory
            .clone()
            .map(|unsaved_directory| Reader {
                unsaved_directory,
                jobs: self.jobs.clone(),
            })
    }

    /// Sends `change` to the thread, which lives as long as the way to it,
    /// so that sending cannot fail
    fn send(&self, change: Change) {
        let _ = self.jobs.send(Job::Change(change));
    }

    /// Has the thread run `then` once it has made every change sent before
    pub(super) fn after(&self, then: impl FnOnce() + Send + 'static) {
        let _ = self.jobs.send(Job::After(Box::new(then)));
    }

    /// Waits for the thread to make every change sent to it so far
    pub(super) fn settled(&self) {
        let (done, settled) = mpsc::channel();
        self.after(move || {
            let _ = done.send(());
        });
        let _ = settled.recv();
    }
}

impl Drop for Keeper {
    /// Waits for the thread to make every change sent to it, so that
    /// closing the window loses none
    fn drop(&mut self) {
        self.settled();
    }
}

/// Way to read the kept files that the keeper writes.  A file read straight
/// from the disk can lack changes that the thread has yet to make, which
/// brings back older text than the keeper was given, so the thread reads it
/// after every change sent before the asking.
#[derive(Clone)]
pub struct Reader {
    unsaved_directory: PathBuf,
    jobs: mpsc::Sender<Job>,
}

impl Reader {
    /// Returns the unsaved changes kept for the show in `file`, or for a new
    /// show without one, or none when nothing is kept.  Waits for the
    /// thread of the keeper, so it belongs on a thread of its own.
    pub(super) fn read(&self, file: Option<&Path>) -> Result<Option<String>, String> {
        self.asked(file)
            .recv()
            .unwrap_or_else(|_| Err("the keeper of unsaved changes has stopped".to_owned()))
    }

    /// Asks the thread for the changes kept for the show in `file`, and
    /// returns the way to its answer
    pub(super) fn asked(
        &self,
        file: Option<&Path>,
    ) -> mpsc::Receiver<Result<Option<String>, String>> {
        let kept = kept_file(&self.unsaved_directory, file);
        let (reply, answer) = mpsc::channel();
        let _ = self.jobs.send(Job::After(Box::new(move || {
            let _ = reply.send(read(&kept));
        })));
        answer
    }
}

type Then = Box<dyn FnOnce() + Send>;

enum Job {
    Change(Change),
    /// Runs once every change sent before it is made
    After(Then),
}

enum Change {
    Write(PathBuf, String),
    Remove(PathBuf),
}

impl Change {
    fn file(&self) -> &Path {
        match self {
            Change::Write(file, _) | Change::Remove(file) => file,
        }
    }

    /// A failure names the show by its kept file, which goes by the name of
    /// the show's file.  A write does not wait for the disk, since a kept
    /// file only has to outlast a crash of the program.
    fn made(&self) -> Result<(), String> {
        match self {
            Change::Write(file, text) => write(file, text, false),
            Change::Remove(file) => remove(file),
        }
        .map_err(|error| {
            let name = self.file().file_name().unwrap_or_default().display();
            format!("Cannot keep the unsaved changes to {name}: {error}")
        })
    }
}

/// Each turn takes every job that waits, makes only the newest change to
/// each file, in the order of those changes, and then runs what is to run
/// after them.
fn work(jobs: &mpsc::Receiver<Job>, failed: &mpsc::Sender<String>) {
    jobs.iter().for_each(|first| {
        let (changes, afters): (Vec<_>, Vec<_>) = iter::once(first)
            .chain(jobs.try_iter())
            .partition_map(|job| match job {
                Job::Change(change) => Either::Left(change),
                Job::After(then) => Either::Right(then),
            });
        let newest: Vec<Change> = changes
            .into_iter()
            .rev()
            .unique_by(|change| change.file().to_owned())
            .collect();
        newest
            .into_iter()
            .rev()
            .filter_map(|change| change.made().err())
            .for_each(|failure| {
                let _ = failed.send(failure);
            });
        afters.into_iter().for_each(|then| then());
    });
}
