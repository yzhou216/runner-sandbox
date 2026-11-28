// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the threads of the devices are asked and what they answer, the hold
//! on a stream, and the devices that they open

use std::sync::mpsc::Sender;

use kira::backend::cpal::cpal::Stream;

use crate::back::audio::{
    Fault,
    patch::{Key, Need},
    sound::mixer::Mixer,
};

/// What a thread is asked to do
pub(super) enum Job {
    /// Open the device of a need, and hand the mixer a way to close it
    Open { need: Need, hold: Sender<Job> },
    /// Tell how many channels the device of a need would open with
    Offers { need: Need },
    /// Close the stream of the identity
    Close(u64),
}

/// What a thread answers
pub(in crate::back::audio) enum Answer {
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
pub(in crate::back::audio) struct Hold {
    pub(super) id: u64,
    pub(super) jobs: Sender<Job>,
}

impl Drop for Hold {
    fn drop(&mut self) {
        let _ = self.jobs.send(Job::Close(self.id));
    }
}

/// Devices that the threads open, which a test replaces with some that need
/// no sound card
pub(in crate::back::audio) trait Make: Send + Sync + 'static {
    /// What plays the sound of an open device until it is dropped
    type Stream: Send + 'static;

    /// Opens the device of `need`, and returns the mixer with its stream
    fn open(&self, need: &Need) -> Result<(Mixer, Self::Stream), Fault>;

    fn offers(&self, need: &Need) -> Option<u16>;
}

/// Devices of the machine
pub(super) struct Devices;

impl Make for Devices {
    type Stream = Stream;

    fn open(&self, need: &Need) -> Result<(Mixer, Stream), Fault> {
        Mixer::open(need.key.as_deref(), need.name.as_deref(), need.channels)
    }

    fn offers(&self, need: &Need) -> Option<u16> {
        Mixer::offers(need.key.as_deref(), need.name.as_deref(), need.channels)
    }
}
