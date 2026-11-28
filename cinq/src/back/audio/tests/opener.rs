// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the threads that open the devices, on devices that need no
//! sound card

mod panicking;

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::Duration,
};

use super::{
    fakes::{Gate, MAIN, is_open, named, show},
    settle, until, waiting,
};
use crate::back::{
    Audio, Channels, Command, Fault, OutputEdit, READY_AHEAD, Workspace,
    audio::{
        patch::Need,
        sound::{
            mixer::Mixer,
            opener::{Make, Opener},
        },
    },
};

/// Stream that tells when it closes
struct Stream {
    id: usize,
    events: Arc<Mutex<Vec<String>>>,
}

impl Drop for Stream {
    fn drop(&mut self) {
        note(&self.events, format!("close {}", self.id));
    }
}

fn note(events: &Mutex<Vec<String>>, event: String) {
    events
        .lock()
        .expect("no test panics with the lock")
        .push(event);
}

fn logged(events: &Mutex<Vec<String>>) -> Vec<String> {
    events.lock().expect("no test panics with the lock").clone()
}

/// Devices that open once `gate` lets them and record what happens to them
#[derive(Default)]
struct Recording {
    gate: Gate,
    events: Arc<Mutex<Vec<String>>>,
    next: AtomicUsize,
}

impl Make for Recording {
    type Stream = Stream;

    fn open(&self, need: &Need) -> Result<(Mixer, Stream), Fault> {
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        note(&self.events, format!("open {id}"));
        self.gate.pass();
        note(&self.events, format!("opened {id}"));
        let (mixer, _) = Mixer::bench(need.channels, need.channels, 8000, 16);
        let events = Arc::clone(&self.events);
        Ok((mixer, Stream { id, events }))
    }

    fn offers(&self, _: &Need) -> Option<u16> {
        None
    }
}

/// Returns audio on recording devices that wait at the closed gate, which
/// is returned too, and the events that they note
fn recorded() -> (Audio, Gate, Arc<Mutex<Vec<String>>>) {
    let gate = Gate::closed();
    let recording = Recording {
        gate: gate.clone(),
        ..Recording::default()
    };
    let events = Arc::clone(&recording.events);
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(recording);
    (audio, gate, events)
}

fn on_a(channels: Channels) -> Workspace {
    Workspace::default()
        .apply(Command::EditOutput(MAIN, OutputEdit::Device(named("a"))))
        .apply(Command::EditOutput(MAIN, OutputEdit::Channels(channels)))
}

/// Waits until a device has noted `event`
fn noted(events: &Mutex<Vec<String>>, event: &str) {
    waiting(|| {
        logged(events)
            .iter()
            .any(|noted| noted == event)
            .then_some(())
    });
}

#[test]
fn stream_that_opens_for_a_default_device_that_was_left_closes_when_it_answers() {
    let (mut audio, gate, events) = recorded();
    audio.default = Some("a".to_owned());
    audio.sync(Workspace::default(), READY_AHEAD);
    noted(&events, "open 0");
    audio.route(Some("elsewhere"));
    audio.sync(Workspace::default(), READY_AHEAD);
    gate.release();
    until(&mut audio, |audio| {
        is_open(audio, "elsewhere") && logged(&events).contains(&"close 0".to_owned())
    });
    assert!(!is_open(&audio, "a"));
}

/// Devices of which `a` never answers until it is let go, and `b` opens
struct Hanging {
    release: Mutex<Receiver<()>>,
    asked: Arc<AtomicUsize>,
}

impl Make for Hanging {
    type Stream = ();

    fn open(&self, need: &Need) -> Result<(Mixer, ()), Fault> {
        if need.key.as_deref() == Some("a") {
            self.asked.fetch_add(1, Ordering::SeqCst);
            let _ = self.release.lock().expect("no test panics").recv();
            return Err(Fault::Missing("there is no audio device".to_owned()));
        }
        let (mixer, _) = Mixer::bench(need.channels, need.channels, 8000, 16);
        Ok((mixer, ()))
    }

    fn offers(&self, _: &Need) -> Option<u16> {
        None
    }
}

fn hanging() -> (Audio, Sender<()>, Arc<AtomicUsize>) {
    let (release, waiting) = mpsc::channel();
    let asked = Arc::new(AtomicUsize::new(0));
    let hanging = Hanging {
        release: Mutex::new(waiting),
        asked: Arc::clone(&asked),
    };
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(hanging).patient(Duration::from_millis(500));
    (audio, release, asked)
}

#[test]
fn device_that_never_answers_holds_up_no_other_and_tells_so_after_a_while() {
    let (mut audio, release, asked) = hanging();
    let workspace = show(Channels::MAIN, Channels::Stereo(0, 1));
    audio.sync(workspace.clone(), READY_AHEAD);
    assert_eq!(audio.problems(), [], "nothing is told at first");
    until(&mut audio, |audio| is_open(audio, "b"));
    until(&mut audio, |audio| !audio.problems().is_empty());
    assert_eq!(audio.problems(), [(MAIN, Fault::Unanswered)]);
    // Syncs go on and ask for no second open of the device
    audio.sync(workspace, READY_AHEAD);
    release.send(()).expect("the device waits");
    settle(&mut audio);
    assert!(matches!(
        audio.problems().as_slice(),
        [(MAIN, Fault::Missing(_))]
    ));
    assert_eq!(asked.load(Ordering::SeqCst), 1, "no second open was asked");
}
