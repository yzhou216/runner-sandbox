// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shows and devices that the tests of opening share: devices that fail to
//! open, and audio with mixers that need no sound card

use std::{
    sync::{
        Arc, Condvar, Mutex, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use super::voice::opened;
use crate::back::{
    Audio, Channels, Command, CueId, Device, Fault, OutputEdit, OutputId, Workspace,
    audio::{
        mixer::{Mix, Mixer},
        opener::{Make, Opener},
        patch::{Need, Patch},
    },
};

pub(super) const MAIN: OutputId = OutputId(1);
pub(super) const SUB: OutputId = OutputId(2);

pub(super) fn named(id: &str) -> Device {
    Device::Named {
        id: id.to_owned(),
        name: id.to_owned(),
    }
}

/// Returns a show of Main on the device `a` and Sub on the device `b`, each
/// on the channels that `channels` gives it
pub(super) fn show(main: Channels, sub: Channels) -> Workspace {
    Workspace::default()
        .apply(Command::EditOutput(MAIN, OutputEdit::Device(named("a"))))
        .apply(Command::EditOutput(MAIN, OutputEdit::Channels(main)))
        .apply(Command::AddOutput {
            at: 1,
            count: 1,
            anchor: None,
        })
        .apply(Command::EditOutput(SUB, OutputEdit::Device(named("b"))))
        .apply(Command::EditOutput(SUB, OutputEdit::Channels(sub)))
}

pub(super) fn patch(workspace: &Workspace) -> Patch {
    Patch::of(workspace.outputs(), &None)
}

/// Longest that a thread waits at a closed [`Gate`], which only bounds a test
/// that hangs
const HANG: Duration = Duration::from_secs(10);

/// Barrier that a fake device waits at while a test holds it closed, so that
/// the device is busy exactly as long as the scenario needs, however slow the
/// machine is.  A gate that is open lets every thread through.
#[derive(Clone)]
pub(super) struct Gate(Arc<(Mutex<bool>, Condvar)>);

impl Default for Gate {
    fn default() -> Self {
        Self(Arc::new((Mutex::new(true), Condvar::new())))
    }
}

impl Gate {
    pub(super) fn closed() -> Self {
        Self(Arc::new((Mutex::new(false), Condvar::new())))
    }

    /// Lets the threads that wait at the gate through, now and later
    pub(super) fn release(&self) {
        let (open, waiting) = &*self.0;
        *open.lock().unwrap_or_else(PoisonError::into_inner) = true;
        waiting.notify_all();
    }

    /// Waits until the gate is open
    pub(super) fn pass(&self) {
        let (open, waiting) = &*self.0;
        let _ = waiting.wait_timeout_while(
            open.lock().unwrap_or_else(PoisonError::into_inner),
            HANG,
            |open| !*open,
        );
    }
}

/// Devices that fail to open once `gate` lets them, and that offer `offers`
/// channels, and the counts of the opens and of the looks that they were
/// asked for
#[derive(Clone, Default)]
pub(super) struct Fake {
    pub(super) gate: Gate,
    pub(super) offers: Option<u16>,
    pub(super) opened: Arc<AtomicUsize>,
    pub(super) looked: Arc<AtomicUsize>,
}

impl Make for Fake {
    type Stream = ();

    fn open(&self, _: &Need) -> Result<(Mixer, ()), Fault> {
        self.opened.fetch_add(1, Ordering::SeqCst);
        self.gate.pass();
        Err(Fault::Missing("there is no audio device".to_owned()))
    }

    fn offers(&self, _: &Need) -> Option<u16> {
        self.looked.fetch_add(1, Ordering::SeqCst);
        self.offers
    }
}

/// Returns audio that has opened the devices `a` and `b` of two channels
/// as the show asks, each with its mix, and the sound of a cue on each
/// device and one on both, which opens its devices as `fake` does
pub(super) fn open_on_both(fake: Fake) -> (Audio, Mix, Mix) {
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(fake);
    audio.patch = patch(&show(Channels::MAIN, Channels::Stereo(0, 1)));
    let (mut a, mix_a) = Mixer::bench(2, 2, 8000, 16);
    let (mut b, mix_b) = Mixer::bench(2, 2, 8000, 16);
    let on_a = opened(vec![("a", &mut a)]).expect("the sound starts");
    let on_b = opened(vec![("b", &mut b)]).expect("the sound starts");
    let on_both = opened(vec![("a", &mut a), ("b", &mut b)]).expect("the sound starts");
    audio.voices.insert(CueId(1), on_a);
    audio.voices.insert(CueId(2), on_b);
    audio.voices.insert(CueId(3), on_both);
    audio.mixers.insert(Some("a".to_owned()), Ok(a));
    audio.mixers.insert(Some("b".to_owned()), Ok(b));
    (audio, mix_a, mix_b)
}

/// Renders a mix until its sounds that went have faded out, and returns
/// how many sounds it holds
pub(super) fn sounds(mix: &mut Mix) -> usize {
    mix.render(&mut [0.0_f32; 512]);
    mix.sounds()
}

pub(super) fn is_open(audio: &Audio, device: &str) -> bool {
    matches!(audio.mixers.get(&Some(device.to_owned())), Some(Ok(_)))
}

pub(super) fn cues(audio: &Audio) -> Vec<u32> {
    let mut cues: Vec<u32> = audio.voices.keys().map(|cue| cue.0).collect();
    cues.sort_unstable();
    cues
}
