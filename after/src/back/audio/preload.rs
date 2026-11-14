// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sounds made ready ahead of the runs that start them
//!
//! Opening a sound file and decoding its first moments can wait on the
//! disk, as a drive that has gone to sleep makes it wait.  So a thread of
//! its own opens, ahead of time, the sounds that runs may start next, and
//! begins decoding each from the start of its clip.  Starting one of them
//! then waits on nothing.  A sound made ready that no run wants any more is
//! let go, which ends its decoding.  Edits to a clip replace its ask faster
//! than files open, so the thread opens only the newest ask of each cue.

use std::{collections::HashMap, convert::Infallible, iter, path::PathBuf, sync::mpsc, thread};

use itertools::Itertools;

use kira::sound::{
    FromFileError, Sound, SoundData,
    streaming::{StreamingSoundData, StreamingSoundHandle},
};

use super::voice;
use crate::back::{Clip, CueId, items::taken};

/// Part of a file that a run of an audio or video cue may start
#[derive(Clone, PartialEq)]
pub(super) struct Wanted {
    pub(super) path: PathBuf,
    pub(super) clip: Clip,
}

pub(super) struct Preloaded {
    sound: Box<dyn Sound>,
    handle: StreamingSoundHandle<FromFileError>,
    /// Sample rate of the file, if it has frames
    pub(super) rate: Option<u32>,
}

impl SoundData for Preloaded {
    type Error = Infallible;
    type Handle = StreamingSoundHandle<FromFileError>;

    fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
        Ok((self.sound, self.handle))
    }
}

/// Sound for the thread to make ready
pub(super) struct Ask {
    pub(super) cue: CueId,
    pub(super) wanted: Wanted,
    pub(super) level: f32,
}

type Made = (CueId, Wanted, Result<Preloaded, String>);

pub(super) struct Preloader {
    asks: mpsc::Sender<Ask>,
    made: mpsc::Receiver<Made>,
    ready: HashMap<CueId, (Wanted, Result<Preloaded, String>)>,
    /// Sounds asked for and not made yet, by cue
    asked: HashMap<CueId, Wanted>,
}

impl Preloader {
    /// Starts the thread, which runs until the preloader goes
    pub(super) fn new() -> Self {
        let (asks, waiting) = mpsc::channel::<Ask>();
        let (done, made) = mpsc::channel();
        thread::spawn(move || serve(&waiting, &done));
        Self {
            asks,
            made,
            ready: HashMap::new(),
            asked: HashMap::new(),
        }
    }

    /// Takes in the sounds that the thread has made since the last call,
    /// apart from those that are no longer wanted
    pub(super) fn collect(&mut self) {
        self.made.try_iter().for_each(|(cue, wanted, sound)| {
            if self.asked.get(&cue) == Some(&wanted) {
                self.asked.remove(&cue);
                self.ready.insert(cue, (wanted, sound));
            }
        });
    }

    /// Takes the sound made ready for `cue`, or why it could not be made,
    /// if it is the one that the cue wants now.  A sound made for another
    /// file or clip is let go.
    pub(super) fn take(
        &mut self,
        cue: CueId,
        wanted: &Wanted,
    ) -> Option<Result<Preloaded, String>> {
        taken(&mut self.ready, &cue, |made| made == wanted)
    }

    /// Makes ready the sounds of `upcoming`, apart from those that are ready
    /// or asked for already, and lets go of any other
    pub(super) fn want(&mut self, upcoming: Vec<Ask>) {
        let wants = |cue: &CueId, sound: &Wanted| {
            upcoming
                .iter()
                .any(|ask| ask.cue == *cue && ask.wanted == *sound)
        };
        self.ready.retain(|cue, (made, _)| wants(cue, made));
        self.asked.retain(|cue, asked| wants(cue, asked));
        let missing: Vec<Ask> = upcoming
            .into_iter()
            .filter(|ask| !self.ready.contains_key(&ask.cue) && !self.asked.contains_key(&ask.cue))
            .collect();
        missing.into_iter().for_each(|ask| {
            self.asked.insert(ask.cue, ask.wanted.clone());
            // Fails only once the thread has panicked, after which GO opens
            // every file itself
            let _ = self.asks.send(ask);
        });
    }
}

fn preloaded(wanted: &Wanted, level: f32) -> Result<Preloaded, String> {
    let data = StreamingSoundData::from_file(&wanted.path).map_err(super::reason)?;
    let rate = voice::rate(&data);
    let (sound, handle) = voice::clipped(data, wanted.clip, wanted.clip.start, level)
        .into_sound()
        .map_err(super::reason)?;
    Ok(Preloaded {
        sound,
        handle,
        rate,
    })
}

/// Makes ready the sounds that `waiting` brings and sends them to `done`.
/// Edits replace asks faster than files open, so each round takes all that
/// wait and opens only the last ask of each cue.
pub(super) fn serve(waiting: &mpsc::Receiver<Ask>, done: &mpsc::Sender<Made>) {
    iter::from_fn(|| waiting.recv().ok()).for_each(|first| {
        let round: Vec<Ask> = iter::once(first).chain(waiting.try_iter()).collect();
        let latest: Vec<Ask> = round.into_iter().rev().unique_by(|ask| ask.cue).collect();
        latest.into_iter().rev().for_each(|ask| {
            let sound = preloaded(&ask.wanted, ask.level);
            let _ = done.send((ask.cue, ask.wanted, sound));
        });
    });
}
