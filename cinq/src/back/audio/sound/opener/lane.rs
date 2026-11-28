// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Thread of one audio device, which opens it, looks at what it offers, and
//! closes its streams, one job at a time

use std::{
    collections::HashMap,
    panic::{self, AssertUnwindSafe},
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
};

use super::{Answer, Hold, Job, Make};
use crate::back::audio::Fault;

/// Why an open fails when the driver of the device panics
const DRIVER_FAILED: &str = "the audio driver failed";

/// Returns what `call` returns, or `None` where it panics, so that a driver
/// that fails costs its device one answer and not its thread.  The panic
/// hook has already reported the panic by the time this returns.
fn survived<T>(call: impl FnOnce() -> T) -> Option<T> {
    panic::catch_unwind(AssertUnwindSafe(call)).ok()
}

pub(super) struct Lane<M: Make> {
    make: Arc<M>,
    done: Sender<Answer>,
    streams: HashMap<u64, M::Stream>,
    next: u64,
}

impl<M: Make> Lane<M> {
    pub(super) fn new(make: Arc<M>, done: Sender<Answer>) -> Self {
        Self {
            make,
            done,
            streams: HashMap::new(),
            next: 0,
        }
    }

    /// Does what the jobs ask, in their order, until every sender has gone
    pub(super) fn serve(mut self, jobs: &Receiver<Job>) {
        jobs.iter().for_each(|job| self.run(job));
    }

    fn run(&mut self, job: Job) {
        match job {
            Job::Open { need, hold } => {
                let opened = survived(|| self.make.open(&need))
                    .unwrap_or_else(|| Err(Fault::Refused(DRIVER_FAILED.to_owned())));
                let result = opened.map(|(mixer, stream)| {
                    let id = self.next;
                    self.next += 1;
                    self.streams.insert(id, stream);
                    mixer.held(Hold { id, jobs: hold })
                });
                let _ = self.done.send(Answer::Opened {
                    key: need.key,
                    result,
                });
            }
            Job::Offers { need } => {
                let offered = survived(|| self.make.offers(&need)).flatten();
                let _ = self.done.send(Answer::Offered {
                    key: need.key,
                    offered,
                });
            }
            Job::Close(id) => {
                survived(|| drop(self.streams.remove(&id)));
            }
        }
    }
}
