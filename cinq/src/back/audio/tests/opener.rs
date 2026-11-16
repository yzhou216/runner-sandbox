// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the threads that open the devices, on devices that need no
//! sound card: the order of opens and closes of one device, and a device
//! that never answers

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use super::{
    fakes::{Gate, MAIN, named, show},
    settle,
};
use crate::back::{
    Audio, Channels, Command, Fault, OutputEdit, READY_AHEAD, Workspace,
    audio::{
        mixer::Mixer,
        opener::{Make, Opener},
        patch::Need,
    },
};

/// Stream that tells when it closes
struct Stream {
    id: usize,
    events: Arc<Mutex<Vec<String>>>,
}

impl Drop for Stream {
    fn drop(&mut self) {
        let mut events = self.events.lock().expect("no test panics with the lock");
        events.push(format!("close {}", self.id));
    }
}

/// Devices that open once `gate` lets them and record what happens to them
#[derive(Default)]
struct Recording {
    gate: Gate,
    events: Arc<Mutex<Vec<String>>>,
    next: AtomicUsize,
}

impl Recording {
    fn note(&self, event: String) {
        let mut events = self.events.lock().expect("no test panics with the lock");
        events.push(event);
    }
}

impl Make for Recording {
    type Stream = Stream;

    fn open(&self, need: &Need) -> Result<(Mixer, Stream), Fault> {
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        self.note(format!("open {id}"));
        self.gate.pass();
        self.note(format!("opened {id}"));
        let (mixer, _) = Mixer::bench(need.channels, need.channels, 8000, 16);
        let events = Arc::clone(&self.events);
        Ok((mixer, Stream { id, events }))
    }

    fn offers(&self, _: &Need) -> Option<u16> {
        None
    }
}

fn on_a(channels: Channels) -> Workspace {
    Workspace::default()
        .apply(Command::EditOutput(MAIN, OutputEdit::Device(named("a"))))
        .apply(Command::EditOutput(MAIN, OutputEdit::Channels(channels)))
}

/// Waits until `done` holds, taking in the answers as a sync does, for at
/// most five seconds
fn until(audio: &mut Audio, done: impl Fn(&Audio) -> bool) {
    let by = Instant::now() + Duration::from_secs(5);
    while !done(audio) && Instant::now() < by {
        audio.arrived();
        thread::sleep(Duration::from_millis(5));
    }
}

/// Waits until a device has noted `event`, for at most five seconds
fn noted(events: &Mutex<Vec<String>>, event: &str) {
    let by = Instant::now() + Duration::from_secs(5);
    while !events
        .lock()
        .expect("no test panics with the lock")
        .iter()
        .any(|noted| noted == event)
        && Instant::now() < by
    {
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn stream_that_an_open_made_stale_closes_before_the_next_open_of_its_device() {
    let gate = Gate::closed();
    let recording = Recording {
        gate: gate.clone(),
        ..Recording::default()
    };
    let events = Arc::clone(&recording.events);
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(recording);
    audio.sync(on_a(Channels::MAIN), READY_AHEAD);
    noted(&events, "open 0");
    audio.route(Some("elsewhere"));
    audio.sync(on_a(Channels::MAIN), READY_AHEAD);
    gate.release();
    until(&mut audio, |audio| {
        matches!(audio.mixers.get(&Some("a".to_owned())), Some(Ok(_)))
    });
    let events = events.lock().expect("no test panics with the lock");
    assert_eq!(
        *events,
        ["open 0", "opened 0", "close 0", "open 1", "opened 1"]
    );
}

#[test]
fn open_that_has_yet_to_start_when_the_devices_change_never_starts() {
    let gate = Gate::closed();
    let recording = Recording {
        gate: gate.clone(),
        ..Recording::default()
    };
    let events = Arc::clone(&recording.events);
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(recording);
    audio.sync(on_a(Channels::MAIN), READY_AHEAD);
    noted(&events, "open 0");
    // The open that follows the first route waits behind the first open,
    // and the second route makes it stale before it starts
    audio.route(Some("one"));
    audio.sync(on_a(Channels::MAIN), READY_AHEAD);
    audio.route(Some("two"));
    audio.sync(on_a(Channels::MAIN), READY_AHEAD);
    gate.release();
    settle(&mut audio);
    let events = events.lock().expect("no test panics with the lock");
    assert_eq!(
        *events,
        ["open 0", "opened 0", "close 0", "open 1", "opened 1"],
        "the open between them never starts"
    );
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
    let (mut audio, release, _) = hanging();
    audio.sync(show(Channels::MAIN, Channels::Stereo(0, 1)), READY_AHEAD);
    assert_eq!(audio.problems(), [], "nothing is told at first");
    until(&mut audio, |audio| {
        matches!(audio.mixers.get(&Some("b".to_owned())), Some(Ok(_)))
    });
    until(&mut audio, |audio| !audio.problems().is_empty());
    assert_eq!(audio.problems(), [(MAIN, Fault::Unanswered)]);
    // Syncs go on and ask for no second open of the device
    audio.sync(show(Channels::MAIN, Channels::Stereo(0, 1)), READY_AHEAD);
    release.send(()).expect("the device waits");
    until(&mut audio, |audio| !audio.opener.busy());
    assert!(matches!(
        audio.problems().as_slice(),
        [(MAIN, Fault::Missing(_))]
    ));
}

#[test]
fn device_that_never_answers_is_not_asked_a_second_time() {
    let (mut audio, release, asked) = hanging();
    let workspace = show(Channels::MAIN, Channels::Stereo(0, 1));
    audio.sync(workspace.clone(), READY_AHEAD);
    until(&mut audio, |audio| {
        matches!(audio.mixers.get(&Some("b".to_owned())), Some(Ok(_)))
    });
    until(&mut audio, |audio| !audio.problems().is_empty());
    audio.sync(workspace, READY_AHEAD);
    release.send(()).expect("the device waits");
    until(&mut audio, |audio| !audio.opener.busy());
    assert_eq!(asked.load(Ordering::SeqCst), 1);
}
