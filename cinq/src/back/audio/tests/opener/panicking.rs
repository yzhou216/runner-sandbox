// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A device whose driver panics is refused and answers on the next try, and
//! one whose stream panics as it closes answers still

use std::sync::atomic::{AtomicUsize, Ordering};

use super::on_a;
use crate::back::{
    Audio, Channels, Fault, READY_AHEAD,
    audio::{
        patch::Need,
        sound::{
            mixer::Mixer,
            opener::{Answer, Make, Opener},
        },
        tests::{
            fakes::{MAIN, is_open, overdue},
            settle, until, waiting,
        },
    },
};

/// Devices of which the driver panics on the first open and on the first look,
/// and that then open and offer two channels
#[derive(Default)]
struct Panicking {
    opens: AtomicUsize,
    looks: AtomicUsize,
}

impl Make for Panicking {
    type Stream = ();

    fn open(&self, need: &Need) -> Result<(Mixer, ()), Fault> {
        assert!(self.opens.fetch_add(1, Ordering::SeqCst) > 0, "driver bug");
        let (mixer, _) = Mixer::bench(need.channels, need.channels, 8000, 16);
        Ok((mixer, ()))
    }

    fn offers(&self, _: &Need) -> Option<u16> {
        assert!(self.looks.fetch_add(1, Ordering::SeqCst) > 0, "driver bug");
        Some(2)
    }
}

#[test]
fn device_whose_driver_panics_while_opening_is_refused_and_opens_on_the_next_try() {
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(Panicking::default());
    audio.sync(on_a(Channels::MAIN), READY_AHEAD);
    settle(&mut audio);
    assert_eq!(
        audio.problems(),
        [(MAIN, Fault::Refused("the audio driver failed".to_owned()))]
    );
    overdue(&mut audio);
    audio.revived();
    until(&mut audio, |audio| is_open(audio, "a"));
    assert_eq!(audio.problems(), []);
}

#[test]
fn device_whose_driver_panics_while_asked_what_it_offers_offers_nothing_then_answers() {
    let mut opener = Opener::with(Panicking::default());
    let need = Need {
        key: Some("a".to_owned()),
        name: None,
        channels: 1,
    };
    let offered = |opener: &mut Opener| {
        opener.probe(need.clone());
        waiting(|| match opener.answers().as_slice() {
            [] => None,
            [Answer::Offered { offered, .. }] => Some(*offered),
            _ => panic!("one look gets one answer"),
        })
        .expect("the device answers")
    };
    assert_eq!(offered(&mut opener), None);
    assert_eq!(offered(&mut opener), Some(2));
}

/// Devices whose streams panic as they close
struct Fused;

struct Fuse;

impl Drop for Fuse {
    fn drop(&mut self) {
        panic!("driver bug");
    }
}

impl Make for Fused {
    type Stream = Fuse;

    fn open(&self, need: &Need) -> Result<(Mixer, Fuse), Fault> {
        let (mixer, _) = Mixer::bench(need.channels, need.channels, 8000, 16);
        Ok((mixer, Fuse))
    }

    fn offers(&self, _: &Need) -> Option<u16> {
        Some(2)
    }
}

#[test]
fn device_whose_stream_panics_as_it_closes_goes_on_to_answer() {
    let mut opener = Opener::with(Fused);
    let need = Need {
        key: Some("a".to_owned()),
        name: None,
        channels: 1,
    };
    opener.open(need.clone());
    let opened = waiting(|| opener.answers().pop()).expect("the device opens");
    // Dropping the mixer asks the thread of the device to close its stream
    drop(opened);
    opener.probe(need);
    let offered = waiting(|| opener.answers().pop());
    assert!(offered.is_some(), "the thread of the device still answers");
}
