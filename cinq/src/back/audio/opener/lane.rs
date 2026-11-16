// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Thread of one audio device, which opens it, looks at what it offers, and
//! closes its streams, one job at a time

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
};

use super::{Answer, Hold, Job, Make, lock};

pub(super) struct Lane<M: Make> {
    make: Arc<M>,
    done: Sender<Answer>,
    generation: Arc<Mutex<u64>>,
    streams: HashMap<u64, M::Stream>,
    next: u64,
}

impl<M: Make> Lane<M> {
    pub(super) fn new(make: Arc<M>, done: Sender<Answer>, generation: Arc<Mutex<u64>>) -> Self {
        Self {
            make,
            done,
            generation,
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
            Job::Open {
                need,
                generation,
                hold,
            } if self.current(generation) => {
                let opened = self.make.open(&need);
                // Held through the answer, so that a restart finds it among
                // the answers that it lets go of, or this finds the restart
                let current = lock(&self.generation);
                if *current != generation {
                    // Dropping what opened closes its stream here, ahead of
                    // the next job
                    return;
                }
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
            Job::Offers { need, generation } if self.current(generation) => {
                let offered = self.make.offers(&need);
                let current = lock(&self.generation);
                if *current == generation {
                    let _ = self.done.send(Answer::Offered {
                        key: need.key,
                        offered,
                    });
                }
            }
            Job::Close(id) => {
                self.streams.remove(&id);
            }
            // A job of a past generation that has yet to start
            Job::Open { .. } | Job::Offers { .. } => {}
        }
    }

    fn current(&self, generation: u64) -> bool {
        *lock(&self.generation) == generation
    }
}
