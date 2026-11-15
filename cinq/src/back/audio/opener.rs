// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Thread that opens audio devices, so that the interface never waits on one
//!
//! Opening a device lists what it offers and starts its stream, which can
//! take seconds on a Dante soundcard that restarts, or on the ALSA plugin of
//! PulseAudio, and a device that fails is tried again every two seconds.
//! The thread does that work and answers over a channel, while the main
//! thread goes on with the clock, the stage, and the keys.  The streams stay
//! on the thread that opened them, since a stream cannot cross threads on
//! every platform, and close there once the mixer that holds them goes.

use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use kira::backend::cpal::cpal::Stream;

use super::{
    Fault,
    mixer::Mixer,
    patch::{Key, Need},
};

/// What the thread is asked to do
pub(super) enum Job {
    /// Open the device of a need, and hand the mixer a way to close it
    Open {
        need: Need,
        generation: u64,
        hold: Sender<Job>,
    },
    /// Tell how many channels the device of a need would open with
    Offers { need: Need, generation: u64 },
    /// Close the stream of the identity
    Close(u64),
}

/// What the thread answers
pub(super) enum Answer {
    Opened {
        key: Key,
        result: Result<Mixer, Fault>,
    },
    Offered {
        key: Key,
        offered: Option<u16>,
    },
}

/// Hold on the stream of a mixer, which lives on the thread that opens
/// devices and closes once the mixer, and so this hold, goes
pub(super) struct Hold {
    id: u64,
    jobs: Sender<Job>,
}

impl Drop for Hold {
    fn drop(&mut self) {
        let _ = self.jobs.send(Job::Close(self.id));
    }
}

/// Devices that the thread opens, which a test replaces with some that need
/// no sound card
pub(super) trait Make: Send + 'static {
    /// Opens the device of `need`, and returns the mixer with the identity
    /// of its stream, which [`Make::close`] closes
    fn open(&mut self, need: &Need) -> Result<(Mixer, u64), Fault>;

    fn offers(&mut self, need: &Need) -> Option<u16>;

    fn close(&mut self, id: u64);
}

/// Devices of the machine
#[derive(Default)]
struct Devices {
    streams: HashMap<u64, Stream>,
    next: u64,
}

impl Make for Devices {
    fn open(&mut self, need: &Need) -> Result<(Mixer, u64), Fault> {
        let (mixer, stream) =
            Mixer::open(need.key.as_deref(), need.name.as_deref(), need.channels)?;
        let id = self.next;
        self.next += 1;
        self.streams.insert(id, stream);
        Ok((mixer, id))
    }

    fn offers(&mut self, need: &Need) -> Option<u16> {
        Mixer::offers(need.key.as_deref(), need.name.as_deref(), need.channels)
    }

    fn close(&mut self, id: u64) {
        self.streams.remove(&id);
    }
}

pub(super) struct Opener {
    jobs: Sender<Job>,
    answers: Receiver<(u64, Answer)>,
    /// Number of the sync of devices that the answers belong to, so that
    /// answers about the devices of an audio that has gone are let go
    generation: u64,
    /// Devices being opened, which no second open starts for
    opening: HashSet<Key>,
    /// Devices being asked how many channels they offer
    probing: HashSet<Key>,
}

impl Opener {
    /// Starts the thread, which runs until the opener and every mixer that
    /// it opened have gone
    pub(super) fn new() -> Self {
        Self::with(Devices::default())
    }

    pub(super) fn with(make: impl Make) -> Self {
        let (jobs, waiting) = mpsc::channel();
        let (done, answers) = mpsc::channel();
        thread::spawn(move || serve(make, &waiting, &done));
        Self {
            jobs,
            answers,
            generation: 0,
            opening: HashSet::new(),
            probing: HashSet::new(),
        }
    }

    /// Lets go of what the thread answers for the devices asked for so far,
    /// and forgets that they are being opened, so that a new sync of the
    /// outputs asks again.  The thread goes on, so that the jobs of the new
    /// devices wait for those that run, and no two opens of one device run
    /// together.
    pub(super) fn restart(&mut self) {
        self.generation += 1;
        self.opening.clear();
        self.probing.clear();
    }

    /// Asks the thread to open the device of `need`, unless it is being
    /// opened already
    pub(super) fn open(&mut self, need: Need) {
        if self.opening.insert(need.key.clone()) {
            let job = Job::Open {
                need,
                generation: self.generation,
                hold: self.jobs.clone(),
            };
            let _ = self.jobs.send(job);
        }
    }

    /// Asks the thread how many channels the device of `need` offers,
    /// unless it is being asked already
    pub(super) fn probe(&mut self, need: Need) {
        if self.probing.insert(need.key.clone()) {
            let _ = self.jobs.send(Job::Offers {
                need,
                generation: self.generation,
            });
        }
    }

    /// Returns whether the device of `key` is being opened
    pub(super) fn opening(&self, key: &Key) -> bool {
        self.opening.contains(key)
    }

    /// Returns whether any device is being opened, or asked about
    #[cfg(test)]
    pub(super) fn busy(&self) -> bool {
        !self.opening.is_empty() || !self.probing.is_empty()
    }

    /// Takes in the answers that the thread has given since the last call,
    /// apart from those about the devices of a past generation
    pub(super) fn answers(&mut self) -> Vec<Answer> {
        let answers: Vec<_> = self.answers.try_iter().collect();
        answers
            .into_iter()
            .filter(|(generation, _)| *generation == self.generation)
            .map(|(_, answer)| {
                match &answer {
                    Answer::Opened { key, .. } => self.opening.remove(key),
                    Answer::Offered { key, .. } => self.probing.remove(key),
                };
                answer
            })
            .collect()
    }
}

/// Does what the jobs ask, in their order, until every sender has gone
fn serve(mut make: impl Make, jobs: &Receiver<Job>, done: &Sender<(u64, Answer)>) {
    jobs.iter().for_each(|job| match job {
        Job::Open {
            need,
            generation,
            hold,
        } => {
            let result = make
                .open(&need)
                .map(|(mixer, id)| mixer.held(Hold { id, jobs: hold }));
            let _ = done.send((
                generation,
                Answer::Opened {
                    key: need.key,
                    result,
                },
            ));
        }
        Job::Offers { need, generation } => {
            let offered = make.offers(&need);
            let _ = done.send((
                generation,
                Answer::Offered {
                    key: need.key,
                    offered,
                },
            ));
        }
        Job::Close(id) => make.close(id),
    });
}
