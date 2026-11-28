// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the video output and its decoder, on videos that [`movie`]
//! makes and [`boxes`] changes, and on pictures that [`still`] makes

mod boxes;
mod damaged;
mod decoding;
mod devamp;
mod layouts;
pub(in crate::back) mod movie;
mod player;
mod probing;
mod runs;
mod skipped;
mod still;

use std::{
    iter,
    ops::{ControlFlow, RangeInclusive},
    path::Path,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use super::{
    decoder::{self, Sent},
    probe,
};
use crate::back::{
    Clip, Cue, CueId, Kind, Load, Loop, Picture, Video, Workspace, load_files,
    workspace::tests::{GO, fresh},
};
use boxes::reworked;
use movie::{RATE, picture_number};

/// Longest that a test waits for a thread to decode a picture
const PATIENCE: Duration = Duration::from_secs(10);

/// Returns a workspace of one cue of `kind` that plays the file at `path`
/// and has loaded it
fn showing(path: &Path, kind: Kind) -> Workspace {
    let file = path.file_name().and_then(|name| name.to_str());
    let cue = Cue::new(CueId(1), "1", kind, "Cue").file(file.expect("the file has a name"));
    showing_cues(path, vec![cue.build()])
}

fn showing_cues(path: &Path, cues: Vec<Cue>) -> Workspace {
    let directory = path.parent().expect("the file is in a directory");
    let workspace = fresh(cues).directory(directory);
    let loads = load_files(Load::all(&workspace, workspace.unloaded()));
    loads.into_iter().fold(workspace, Workspace::apply)
}

/// Calls `step` on `state` until it reports that it is done or the patience
/// of the tests runs out, a moment apart, and returns the state.  The steps
/// of these tests wait for a thread to decode, which takes seconds on a busy
/// machine, so recursing would leave a stack frame for every poll and
/// overflow the stack of the test thread, which aborts the whole run instead
/// of failing the one test.
fn patiently<S>(state: S, mut step: impl FnMut(S) -> (S, bool)) -> S {
    let started = Instant::now();
    iter::repeat(())
        .try_fold(state, |state, ()| {
            let (state, done) = step(state);
            if done || started.elapsed() > PATIENCE {
                ControlFlow::Break(state)
            } else {
                thread::sleep(Duration::from_millis(5));
                ControlFlow::Continue(state)
            }
        })
        .break_value()
        .expect("an endless iterator ends only by breaking")
}

/// Syncs `video` with `workspace` until a picture comes due or the
/// workspace no longer runs the cue, and returns the workspace, along with
/// the picture
fn synced(video: &mut Video, workspace: Workspace) -> (Workspace, Option<(CueId, Picture)>) {
    patiently((workspace, None), |(workspace, _)| {
        let (workspace, pictures) = video.sync(workspace);
        let picture = pictures.into_iter().next();
        let done = picture.is_some() || workspace.active().is_empty();
        ((workspace, picture), done)
    })
}

fn clip(start: f64, end: f64, looping: bool) -> Clip {
    Clip {
        start,
        end,
        looping: looping.then_some(Loop {
            start,
            end,
            released: None,
        }),
    }
}

/// Returns the pictures of `numbers`, the first due at `at` pictures of the
/// video, and each after it a picture later
fn from_at(numbers: RangeInclusive<usize>, at: f64) -> Vec<(usize, f64)> {
    let first = *numbers.start();
    numbers
        .map(|number| (number, at + (number - first) as f64))
        .collect()
}

/// Returns what decoding pass `pass` through `clip` of the video at `path`
/// sends, for a player that stays as long as the pass does
fn sent(path: &Path, clip: Clip, pass: u32) -> Vec<Sent> {
    let player = Arc::new(());
    let pass = clip.pass(pass).expect("the clip has the pass");
    decoder::start(path.to_owned(), pass, Arc::downgrade(&player))
        .iter()
        .collect()
}

/// Returns the numbers of the pictures that decoding pass `pass` through
/// `clip` of the video at `path` sends, each with the time it is due at in
/// pictures of the video since the start of the clip, once the pass has
/// ended
fn decoded(path: &Path, clip: Clip, pass: u32) -> Vec<(usize, f64)> {
    let sent = sent(path, clip, pass);
    assert!(matches!(sent.last(), Some(Sent::Ended)), "the pass ends");
    sent.iter()
        .filter_map(|sent| match sent {
            Sent::Picture(frame) => Some((
                picture_number(&frame.picture.pixels),
                frame.at * f64::from(RATE),
            )),
            _ => None,
        })
        .collect()
}

fn same((number, at): (usize, f64), (expected, due): (usize, f64)) -> bool {
    number == expected && (at - due).abs() < 1e-6
}

fn assert_pictures(decoded: &[(usize, f64)], expected: &[(usize, f64)]) {
    let matching =
        decoded.len() == expected.len() && decoded.iter().zip(expected).all(|(&a, &b)| same(a, b));
    assert!(matching, "{decoded:?} is not {expected:?}");
}

/// Returns `bytes` with the body of the box `kind` among the tables of the
/// track replaced by `words`, after the version and flags of a box that
/// holds none of either
fn tabled(bytes: &[u8], kind: &[u8; 4], words: &[u32]) -> Vec<u8> {
    let path = [b"moov", b"trak", b"mdia", b"minf", b"stbl", kind];
    reworked(bytes, &path, &|_| {
        iter::once(0)
            .chain(words.iter().copied())
            .flat_map(u32::to_be_bytes)
            .collect()
    })
}
