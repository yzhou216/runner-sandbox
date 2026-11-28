// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which devices open again as the outputs change, on mixers
//! without devices: the devices that the threads of the devices open are
//! replaced with some that fail, which shows where a device was opened
//! again, and that a sync never waits on one

use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use super::{
    fakes::{Fake, Gate, SUB, cues, is_open, named, open_on_both, overdue, patch, show, sounds},
    settle,
};
use crate::back::{
    Audio, Channels, Command, Fault, OutputEdit, READY_AHEAD, Workspace,
    audio::{
        devices::fault::RETRY,
        sound::{mixer::Mixer, opener::Opener},
    },
};

#[test]
fn outputs_that_still_fit_their_devices_keep_them_open_and_their_sounds() {
    let (mut audio, mut mix_a, mut mix_b) = open_on_both(Fake::default());
    audio.patched(patch(&show(Channels::Stereo(0, 1), Channels::Mono(1))));
    settle(&mut audio);
    assert!(is_open(&audio, "a") && is_open(&audio, "b"));
    assert_eq!(cues(&audio), [1, 2, 3]);
    assert_eq!((sounds(&mut mix_a), sounds(&mut mix_b)), (2, 2));
}

#[test]
fn output_that_needs_more_channels_opens_its_device_again_and_no_other() {
    let (mut audio, mut mix_a, mut mix_b) = open_on_both(Fake::default());
    // Channels 12 and 13 ask a device of two for more, as typing 12-13
    // would, which opens only the device of Sub
    audio.patched(patch(&show(Channels::MAIN, Channels::Stereo(11, 12))));
    settle(&mut audio);
    assert!(is_open(&audio, "a"));
    assert!(matches!(
        audio.mixers.get(&Some("b".to_owned())),
        Some(Err(Fault::Missing(_)))
    ));
    assert_eq!(cues(&audio), [1, 3], "only sounds on device a play on");
    assert_eq!(sounds(&mut mix_a), 2, "the sound of cue 2 goes");
    assert_eq!(sounds(&mut mix_b), 0);
}

#[test]
fn output_that_moves_to_another_device_closes_the_one_it_left() {
    let (mut audio, mut mix_a, _) = open_on_both(Fake::default());
    let workspace = show(Channels::MAIN, Channels::Stereo(0, 1))
        .apply(Command::EditOutput(SUB, OutputEdit::Device(named("a"))));
    audio.patched(patch(&workspace));
    settle(&mut audio);
    assert!(is_open(&audio, "a"));
    assert!(!audio.mixers.contains_key(&Some("b".to_owned())));
    assert_eq!(cues(&audio), [1, 3]);
    assert_eq!(sounds(&mut mix_a), 2);
}

#[test]
fn the_retry_waits_from_the_answer_of_an_open_not_from_asking_for_it() {
    let gate = Gate::closed();
    let (mut audio, ..) = open_on_both(Fake {
        gate: gate.clone(),
        ..Fake::default()
    });
    overdue(&mut audio);
    audio.patched(patch(&show(Channels::MAIN, Channels::Stereo(11, 12))));
    assert!(audio.tried.elapsed() >= RETRY, "the open has not answered");
    gate.release();
    settle(&mut audio);
    assert!(audio.tried.elapsed() < RETRY);
}

#[test]
fn device_that_opened_with_fewer_channels_opens_again_once_it_offers_more() {
    let wide = show(Channels::Stereo(2, 3), Channels::Stereo(0, 1));
    let short = |offers| {
        let (mut audio, ..) = open_on_both(Fake {
            offers,
            ..Fake::default()
        });
        let (a, _) = Mixer::bench(2, 4, 8000, 16);
        audio.mixers.insert(Some("a".to_owned()), Ok(a));
        audio.patch = patch(&wide);
        overdue(&mut audio);
        audio.revived();
        settle(&mut audio);
        audio
    };
    assert!(
        is_open(&short(Some(2)), "a"),
        "it offers no more than it opened with"
    );
    assert!(is_open(&short(None), "a"), "it is not there to look at");
    assert!(!is_open(&short(Some(4)), "a"), "it offers all four now");
}

#[test]
fn sync_never_waits_on_a_device_and_opens_each_device_once_at_a_time() {
    let gate = Gate::closed();
    let fake = Fake {
        gate: gate.clone(),
        ..Fake::default()
    };
    let opened = Arc::clone(&fake.opened);
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(fake);
    let began = Instant::now();
    let workspace = show(Channels::MAIN, Channels::Stereo(0, 1));
    audio.sync(workspace, READY_AHEAD);
    // Retry that falls due while the first open runs starts no second
    overdue(&mut audio);
    audio.sync(show(Channels::MAIN, Channels::Stereo(0, 1)), READY_AHEAD);
    // Gate holds the opens for as long as a hang lasts, far above this
    assert!(began.elapsed() < Duration::from_secs(5));
    assert_eq!(audio.problems(), [], "nothing is told while they open");
    gate.release();
    settle(&mut audio);
    assert_eq!(opened.load(Ordering::SeqCst), 2, "one open for each device");
    assert_eq!(audio.problems().len(), 2, "both tell once they answer");
}

#[test]
fn moving_copying_cutting_and_pasting_outputs_open_only_the_devices_that_changed() {
    let fake = Fake::default();
    let opened = Arc::clone(&fake.opened);
    let (mut audio, mut mix_a, mut mix_b) = open_on_both(fake);
    let change = |audio: &mut Audio, workspace: &Workspace| {
        audio.patched(patch(workspace));
        settle(audio);
    };
    let moved = show(Channels::MAIN, Channels::Stereo(0, 1)).apply(Command::MoveOutput {
        output: SUB,
        by: -1,
    });
    change(&mut audio, &moved);
    let sub = moved.output(SUB).cloned().expect("the show has Sub");
    let copied = moved.apply(Command::PasteOutputs {
        at: 0,
        outputs: vec![sub.clone()],
        restore: false,
        anchor: None,
    });
    change(&mut audio, &copied);
    assert_eq!(opened.load(Ordering::SeqCst), 0, "no device opens again");
    assert_eq!(cues(&audio), [1, 2, 3]);
    assert_eq!((sounds(&mut mix_a), sounds(&mut mix_b)), (2, 2));
    // Cutting both outputs on device b closes it, and device a plays on
    let copy = copied.outputs()[0].id;
    let cut = copied.apply(Command::DeleteOutputs(vec![SUB, copy]));
    change(&mut audio, &cut);
    assert_eq!(opened.load(Ordering::SeqCst), 0);
    assert!(is_open(&audio, "a") && !audio.mixers.contains_key(&Some("b".to_owned())));
    assert_eq!(cues(&audio), [1, 3]);
    // Pasting one back opens device b alone
    let back = cut.apply(Command::PasteOutputs {
        at: 1,
        outputs: vec![sub],
        restore: true,
        anchor: None,
    });
    change(&mut audio, &back);
    assert_eq!(opened.load(Ordering::SeqCst), 1, "only device b opens");
    assert!(is_open(&audio, "a"));
    assert_eq!(cues(&audio), [1, 3]);
    assert_eq!(sounds(&mut mix_a), 2);
}

#[test]
fn pasting_an_output_on_a_device_that_cannot_play_does_not_try_it_again() {
    let fake = Fake::default();
    let opened = Arc::clone(&fake.opened);
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(fake);
    let workspace = show(Channels::MAIN, Channels::Stereo(0, 1));
    audio.patch = patch(&workspace);
    let (a, _) = Mixer::bench(2, 2, 8000, 16);
    audio.mixers.insert(Some("a".to_owned()), Ok(a));
    let fault = Fault::Unplayable("no samples that cinq plays".to_owned());
    audio.mixers.insert(Some("b".to_owned()), Err(fault));
    let sub = workspace.output(SUB).cloned().expect("the show has Sub");
    let copied = workspace.apply(Command::PasteOutputs {
        at: 2,
        outputs: vec![sub],
        restore: false,
        anchor: None,
    });
    audio.patched(patch(&copied));
    settle(&mut audio);
    assert_eq!(opened.load(Ordering::SeqCst), 0);
    assert!(matches!(
        audio.mixers.get(&Some("b".to_owned())),
        Some(Err(Fault::Unplayable(_)))
    ));
}
