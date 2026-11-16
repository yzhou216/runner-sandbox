// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Threads that open audio devices, so that the interface never waits on one
//!
//! Opening a device lists what it offers and starts its stream, which can
//! take seconds on a Dante soundcard that restarts, or on the ALSA plugin of
//! PulseAudio, and a device that fails is tried again every two seconds.
//! A thread for each device does that work and answers over a channel,
//! while the main thread goes on with the clock, the stage, and the keys,
//! and a device that never answers, as an ASIO driver that waits for a
//! dialog, holds up its own thread alone.  A thread takes the jobs of its
//! device in order, so that no two opens of one device run together, and it
//! closes a stream that a newer open has made stale before it takes the
//! next job.  The streams stay on the thread that opened them, and close
//! there once the mixer that holds them goes, since closing a stream can
//! wait on the thread of its device, which the interface must not.

mod job;
mod lane;

use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use super::patch::{Key, Need};
pub(super) use job::{Answer, Hold, Make};
use job::{Devices, Job};
use lane::Lane;

/// Time that an open has to answer before its output tells that the device
/// does not
const PATIENCE: Duration = Duration::from_secs(5);

/// Starts the thread of a device, which takes the jobs that it receives,
/// and answers on `done`, until every sender of its jobs has gone
type Spawn = Box<dyn Fn(Receiver<Job>, Sender<Answer>, Arc<Mutex<u64>>) + Send>;

pub(super) struct Opener {
    spawn: Spawn,
    /// Sender of the jobs of the thread of each device that has had one
    lanes: HashMap<Key, Sender<Job>>,
    done: Sender<Answer>,
    answers: Receiver<Answer>,
    /// Number of the sync of devices that new jobs belong to, which the
    /// threads read to tell that a job is stale, shared so that a thread
    /// answers or finds itself stale as a whole, never between the two
    shared: Arc<Mutex<u64>>,
    generation: u64,
    /// Devices being opened, which no second open starts for, and since
    /// when
    opening: HashMap<Key, Instant>,
    /// Devices being asked how many channels they offer
    probing: HashSet<Key>,
    patience: Duration,
}

impl Opener {
    /// Starts nothing yet: the thread of a device starts with its first
    /// job, and runs until the opener and every mixer that it opened have
    /// gone
    pub(super) fn new() -> Self {
        Self::with(Devices)
    }

    pub(super) fn with<M: Make>(make: M) -> Self {
        let make = Arc::new(make);
        let (done, answers) = mpsc::channel();
        Self {
            spawn: Box::new(move |jobs, done, generation| {
                let lane = Lane::new(Arc::clone(&make), done, generation);
                thread::spawn(move || lane.serve(&jobs));
            }),
            lanes: HashMap::new(),
            done,
            answers,
            shared: Arc::new(Mutex::new(0)),
            generation: 0,
            opening: HashMap::new(),
            probing: HashSet::new(),
            patience: PATIENCE,
        }
    }

    /// Returns the opener that tells after `patience` that an open does not
    /// answer
    #[cfg(test)]
    pub(super) fn patient(self, patience: Duration) -> Self {
        Self { patience, ..self }
    }

    /// Lets go of what the threads answer for the devices asked for so far,
    /// and forgets that they are being opened, so that a new sync of the
    /// outputs asks again.  The threads go on: a job that has yet to start
    /// is dropped, and a stream that an open then makes is closed at once,
    /// before the next open of its device, so that a device that one
    /// program can open at a time never sees two.
    pub(super) fn restart(&mut self) {
        {
            let mut shared = lock(&self.shared);
            *shared += 1;
            self.generation = *shared;
        }
        // Answers that came before the bump are those of the old devices,
        // and no thread sends another until its job is of the new
        // generation, so that their mixers close ahead of the next open
        self.answers.try_iter().for_each(drop);
        self.opening.clear();
        self.probing.clear();
    }

    /// Asks the thread of the device of `need` to open it, unless it is
    /// being opened already
    pub(super) fn open(&mut self, need: Need) {
        if self.opening.contains_key(&need.key) {
            return;
        }
        self.opening.insert(need.key.clone(), Instant::now());
        let generation = self.generation;
        let lane = self.lane(&need.key);
        let job = Job::Open {
            need,
            generation,
            hold: lane.clone(),
        };
        let _ = lane.send(job);
    }

    /// Asks the thread of the device of `need` how many channels it offers,
    /// unless it is being asked already
    pub(super) fn probe(&mut self, need: Need) {
        if self.probing.insert(need.key.clone()) {
            let job = Job::Offers {
                generation: self.generation,
                need: need.clone(),
            };
            let _ = self.lane(&need.key).send(job);
        }
    }

    /// Returns whether the device of `key` is being opened
    pub(super) fn opening(&self, key: &Key) -> bool {
        self.opening.contains_key(key)
    }

    /// Returns whether the device of `key` has been opening for longer than
    /// it has to answer
    pub(super) fn unanswered(&self, key: &Key) -> bool {
        self.opening
            .get(key)
            .is_some_and(|since| since.elapsed() >= self.patience)
    }

    /// Returns whether any device is being opened, or asked about
    #[cfg(test)]
    pub(super) fn busy(&self) -> bool {
        !self.opening.is_empty() || !self.probing.is_empty()
    }

    /// Takes in the answers that the threads have given since the last call
    pub(super) fn answers(&mut self) -> Vec<Answer> {
        self.answers
            .try_iter()
            .inspect(|answer| match answer {
                Answer::Opened { key, .. } => {
                    self.opening.remove(key);
                }
                Answer::Offered { key, .. } => {
                    self.probing.remove(key);
                }
            })
            .collect()
    }

    fn lane(&mut self, key: &Key) -> &Sender<Job> {
        self.lanes.entry(key.clone()).or_insert_with(|| {
            let (jobs, waiting) = mpsc::channel();
            (self.spawn)(waiting, self.done.clone(), Arc::clone(&self.shared));
            jobs
        })
    }
}

/// Takes the lock on a generation, which a thread that panicked leaves a
/// number that is still right
fn lock(generation: &Mutex<u64>) -> MutexGuard<'_, u64> {
    generation.lock().unwrap_or_else(PoisonError::into_inner)
}
