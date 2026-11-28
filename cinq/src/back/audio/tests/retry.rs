// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which failed devices are tried again, and what a retry keeps:
//! the devices that the threads of the devices open are replaced with some
//! that fail

use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use tempfile::TempDir;

use super::{
    ask, collecting,
    fakes::{Fake, open_on_both, overdue, patch, show, sounds},
    settle, silent_file,
};
use crate::back::{
    Audio, Channels, CueId, Fault,
    audio::{
        Wanted,
        devices::fault::RETRY,
        sound::{mixer::Mixer, opener::Opener},
    },
};

/// Makes the sound of cue 1 ready in `audio`, waiting five seconds at most
fn make_ready(audio: &mut Audio, wanted: &Wanted) {
    audio.preloader.want(vec![ask(wanted)]);
    collecting(&mut audio.preloader, |preloader| preloader.ready(CueId(1)))
        .expect("the sound is made ready within five seconds");
}

/// Returns audio on the show of Main on the device `a` and Sub on the
/// device `b`, whose devices have failed as `faults` say, with the sound of
/// a cue made ready, and the file of the sound
fn failed(fake: Fake, faults: [Fault; 2]) -> (Audio, TempDir) {
    let (directory, wanted) = silent_file();
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(fake);
    audio.patch = patch(&show(Channels::MAIN, Channels::Stereo(0, 1)));
    let [a, b] = faults;
    audio.mixers.insert(Some("a".to_owned()), Err(a));
    audio.mixers.insert(Some("b".to_owned()), Err(b));
    make_ready(&mut audio, &wanted);
    overdue(&mut audio);
    (audio, directory)
}

fn missing() -> Fault {
    Fault::Missing("there is no audio device".to_owned())
}

#[test]
fn retrying_devices_that_are_not_there_keeps_the_sounds_made_ready() {
    let fake = Fake::default();
    let opened = Arc::clone(&fake.opened);
    let (mut audio, _directory) = failed(fake, [missing(), missing()]);
    assert!(audio.preloader.ready(CueId(1)), "the sound is made ready");
    audio.revived();
    settle(&mut audio);
    assert_eq!(opened.load(Ordering::SeqCst), 2, "both devices were tried");
    assert!(audio.preloader.ready(CueId(1)), "no run stopped for it");
}

#[test]
fn retrying_a_device_that_a_run_plays_on_keeps_the_sound_made_ready_for_its_next_go() {
    let (mut audio, mut mix_a, _) = open_on_both(Fake::default());
    let (directory, wanted) = silent_file();
    make_ready(&mut audio, &wanted);
    drop(directory);
    audio.patched(patch(&show(Channels::MAIN, Channels::Stereo(11, 12))));
    settle(&mut audio);
    assert!(
        audio.preloader.ready(CueId(1)),
        "the sound that ended is not the one made ready"
    );
    assert_eq!(sounds(&mut mix_a), 2);
}

/// Returns how many opens a retry asks for when the devices `a` and `b`
/// have failed as `faults` say
fn retried(faults: [Fault; 2]) -> usize {
    let fake = Fake::default();
    let opened = Arc::clone(&fake.opened);
    let (mut audio, _directory) = failed(fake, faults);
    audio.revived();
    settle(&mut audio);
    opened.load(Ordering::SeqCst)
}

#[test]
fn only_faults_that_may_pass_are_tried_again() {
    let unplayable = || Fault::Unplayable("no samples that cinq plays".to_owned());
    let refused = Fault::Refused("the device is busy".to_owned());
    // A fault that the show must mend is never tried again
    assert_eq!(retried([unplayable(), unplayable()]), 0);
    assert_eq!(retried([missing(), unplayable()]), 1);
    assert_eq!(retried([refused, Fault::Stopped("it went".to_owned())]), 2);
}

#[test]
fn device_that_offers_no_more_is_looked_at_ever_more_seldom() {
    let fake = Fake {
        offers: Some(2),
        ..Fake::default()
    };
    let looked = Arc::clone(&fake.looked);
    let (mut audio, ..) = open_on_both(fake);
    let key = Some("a".to_owned());
    let (a, _) = Mixer::bench(2, 4, 8000, 16);
    audio.mixers.insert(key.clone(), Ok(a));
    audio.patch = patch(&show(Channels::Stereo(2, 3), Channels::Stereo(0, 1)));
    audio.probes.opened(key.clone(), 2);
    let looks = |audio: &mut Audio, age: Duration| {
        audio.probes.aged(&key, age);
        audio.revived();
        settle(audio);
        looked.load(Ordering::SeqCst)
    };
    assert_eq!(looks(&mut audio, RETRY / 2), 0, "too soon after opening");
    assert_eq!(looks(&mut audio, RETRY), 1);
    assert_eq!(looks(&mut audio, RETRY), 1, "its wait is four seconds now");
    assert_eq!(looks(&mut audio, 2 * RETRY), 2);
}

#[test]
fn interface_looks_again_every_retry_while_a_device_has_failed_in_a_way_that_may_pass() {
    let unplayable = || Fault::Unplayable("no samples that cinq plays".to_owned());
    let (idle, _) = failed(Fake::default(), [unplayable(), unplayable()]);
    assert_eq!(idle.checks(), None, "the show must mend it");
    let (one, _) = failed(Fake::default(), [missing(), unplayable()]);
    assert_eq!(one.checks(), Some(RETRY));
    let (fine, ..) = open_on_both(Fake::default());
    assert_eq!(fine.checks(), None, "every device is open");
}
