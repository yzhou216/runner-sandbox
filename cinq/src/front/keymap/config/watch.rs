// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Watching the configuration file, so that the keys follow it while the
//! program runs
//!
//! Editors often save a file by writing another and renaming it over the
//! first, which ends any watch on the file itself, so the watcher watches
//! the directory that holds the file instead.  While that directory is
//! missing, it watches the nearest one above that exists, so that making
//! the directories and then the file comes through as well.  Each burst of
//! changes reads the file once, after it settles, and formats it first, as
//! [`format`](mod@format) tells, and each reading writes out the problems
//! with the file, which go to standard error in the program, as [`errors`]
//! tells.
//!
//! The watch waits for changes on iced's executor, whose workers keep the
//! clock ticking, and leaves reading and writing to threads: each reading
//! of the file runs on a thread of its own, and a thread of the watch's own
//! writes out the problems, since a write to standard error can wait
//! without end, as it does while a terminal is paused with Control+s.

mod errors;
mod format;
#[cfg(test)]
mod tests;

use std::{
    iter,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use iced::{
    futures::{
        SinkExt, Stream, StreamExt,
        channel::{mpsc, oneshot},
        future,
    },
    stream,
};
use notify_debouncer_full::{
    DebounceEventResult, Debouncer, RecommendedCache, new_debouncer,
    notify::{self, RecommendedWatcher, RecursiveMode, Watcher as _, WatcherKind},
};

use super::{Config, Problem, load};
use errors::warner;
pub use errors::{Errors, guard, restore};

/// Time that changes must settle for before the file is read again, which
/// lets an editor finish saving
const SETTLE: Duration = Duration::from_millis(200);

type Watcher = Debouncer<RecommendedWatcher, RecommendedCache>;

/// Returns the configuration that `file` holds once the watch is in place,
/// so that no change made before goes unseen, and again after each change
/// to the file, and hands each to the thread that writes out its problems
/// to `errors`.  When the file cannot be watched, the configuration comes
/// once more, with a problem that tells why the changes do not apply.
pub fn watch(file: PathBuf, errors: Errors) -> impl Stream<Item = Config> {
    let warnings = warner(errors);
    stream::channel(1, async move |mut output| {
        if let Err(error) = follow(&file, &mut output).await
            && let Some(config) = loaded(&file, false).await
        {
            let problem = format!("is not watched, so changes apply after a restart: {error}");
            let problems = [config.problems, vec![Problem::new(problem)]].concat();
            let _ = output.send(Config { problems, ..config }).await;
        }
    })
    .inspect(move |config| {
        // Thread lives as long as the way to it, so that sending cannot fail
        let _ = warnings.send(config.clone());
    })
}

async fn loaded(file: &Path, changed: bool) -> Option<Config> {
    let (sender, receiver) = oneshot::channel();
    let file = file.to_owned();
    thread::spawn(move || {
        if changed {
            format::reformat(&file);
        }
        // Receiver has gone only when the watch has
        let _ = sender.send(load(Some(&file)));
    });
    receiver.await.ok()
}

async fn follow(file: &Path, output: &mut mpsc::Sender<Config>) -> notify::Result<()> {
    let (sender, mut events) = mpsc::unbounded();
    let mut watcher = new_debouncer(SETTLE, None, move |result| {
        // Channel closes only once the stream has gone
        let _ = sender.unbounded_send(result);
    })?;
    let mut watched = None;
    // First reading follows the start, which leaves the file as it was
    let mut changed = false;
    loop {
        watched = rewatch(&mut watcher, watched.as_deref(), file)?;
        let names = names(file, watched.as_deref());
        if let Some(config) = loaded(file, changed).await {
            let _ = output.send(config).await;
        }
        let touched = (&mut events)
            .any(|result| future::ready(touches(&result, &names)))
            .await;
        if !touched {
            return Ok(());
        }
        changed = true;
    }
}

/// Returns the paths that changes to `file` can come by: its own, and the
/// one past every symbolic link of `watched`, the directory on the way to it
/// that is watched, which is how FSEvents on Darwin reports them
fn names(file: &Path, watched: Option<&Path>) -> Vec<PathBuf> {
    let real = watched.and_then(|directory| {
        let rest = file.strip_prefix(directory).ok()?;
        directory.canonicalize().ok().map(|real| real.join(rest))
    });
    iter::once(file.to_owned()).chain(real).collect()
}

/// Returns whether a burst of changes touches the file that goes by
/// `names`, or a directory on the way to it.  Opening a file, which reading
/// it does, changes nothing.  A watch that failed may have missed a change,
/// so it counts as touching the file.
fn touches(result: &DebounceEventResult, names: &[PathBuf]) -> bool {
    match result {
        Ok(events) => events
            .iter()
            .filter(|event| !event.kind.is_access())
            .flat_map(|event| &event.paths)
            .any(|path| names.iter().any(|name| name.starts_with(path))),
        Err(_) => true,
    }
}

/// Watches the nearest directory on the way to `file` in place of the
/// directory `watched`, and with kqueue the file itself as well, which an
/// editor may write in place, and returns the directory now watched, or why
/// no directory can be watched.  The watch goes in anew even when the
/// directory is the same, since a directory that was deleted and made again
/// takes its old watch with it.
fn rewatch(
    watcher: &mut Watcher,
    watched: Option<&Path>,
    file: &Path,
) -> notify::Result<Option<PathBuf>> {
    // Directory that has gone took its watch with it, so a failure is moot
    if let Some(watched) = watched {
        let _ = watcher.unwatch(watched);
    }
    let directory = watch_nearest(file, |directory| {
        watcher.watch(directory, RecursiveMode::NonRecursive)
    })?;
    if RecommendedWatcher::kind() == WatcherKind::Kqueue {
        // kqueue tells of a directory only the entries that come and go, and
        // not of a file in it written in place.  A file that is missing, or
        // that another has replaced, comes and goes through the watch on its
        // directory, so a failure is moot.
        let _ = watcher.unwatch(file);
        let _ = watcher.watch(file, RecursiveMode::NonRecursive);
    }
    Ok(directory)
}

/// Calls `add` on the nearest directory on the way to `file` that exists,
/// and returns it once watched.  A directory can go between the check that
/// it exists and the watch, with no event to tell of it, so a watch that
/// fails on a directory gone by then falls back to the next one up, while
/// any other failure is an error.
fn watch_nearest(
    file: &Path,
    mut add: impl FnMut(&Path) -> notify::Result<()>,
) -> notify::Result<Option<PathBuf>> {
    file.ancestors()
        .skip(1)
        .filter(|directory| directory.is_dir())
        .find_map(|directory| match add(directory) {
            Err(_) if !directory.is_dir() => None,
            result => Some(result.map(|()| directory.to_owned())),
        })
        .transpose()
}
