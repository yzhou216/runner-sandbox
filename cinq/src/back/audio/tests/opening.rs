// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which devices open again as the outputs change, on mixers
//! without devices: the devices that the thread of the devices opens are
//! replaced with some that fail, which shows where a device was opened
//! again, and that a sync never waits on one

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use super::{settle, voice::opened};
use crate::back::{
    Audio, Channels, Command, CueId, Device, Fault, OutputEdit, OutputId, READY_AHEAD, Workspace,
    audio::{
        mixer::{Mix, Mixer},
        opener::{Make, Opener},
        opening::RETRY,
        patch::{Need, Patch},
    },
};

const MAIN: OutputId = OutputId(1);
const SUB: OutputId = OutputId(2);

fn named(id: &str) -> Device {
    Device::Named {
        id: id.to_owned(),
        name: id.to_owned(),
    }
}

/// Returns a show of Main on the device `a` and Sub on the device `b`, each
/// on the channels that `channels` gives it
fn show(main: Channels, sub: Channels) -> Workspace {
    Workspace::default()
        .apply(Command::EditOutput(MAIN, OutputEdit::Device(named("a"))))
        .apply(Command::EditOutput(MAIN, OutputEdit::Channels(main)))
        .apply(Command::AddOutput { at: 1, count: 1 })
        .apply(Command::EditOutput(SUB, OutputEdit::Device(named("b"))))
        .apply(Command::EditOutput(SUB, OutputEdit::Channels(sub)))
}

fn patch(workspace: &Workspace) -> Patch {
    Patch::of(workspace.outputs(), &None)
}

/// Devices that fail to open, after `delay`, and that offer `offers`
/// channels, and the count of the opens that they were asked for
#[derive(Clone, Default)]
struct Fake {
    delay: Duration,
    offers: Option<u16>,
    opened: Arc<AtomicUsize>,
}

impl Make for Fake {
    fn open(&mut self, _: &Need) -> Result<(Mixer, u64), Fault> {
        self.opened.fetch_add(1, Ordering::SeqCst);
        thread::sleep(self.delay);
        Err(Fault::Missing("there is no audio device".to_owned()))
    }

    fn offers(&mut self, _: &Need) -> Option<u16> {
        self.offers
    }

    fn close(&mut self, _: u64) {}
}

/// Returns audio that has opened the devices `a` and `b` of two channels
/// as the show asks, each with its mix, and the sound of a cue on each
/// device and one on both, which opens its devices as `fake` does
fn open_on_both(fake: Fake) -> (Audio, Mix, Mix) {
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
fn sounds(mix: &mut Mix) -> usize {
    mix.render(&mut [0.0_f32; 512]);
    mix.sounds()
}

fn is_open(audio: &Audio, device: &str) -> bool {
    matches!(audio.mixers.get(&Some(device.to_owned())), Some(Ok(_)))
}

fn cues(audio: &Audio) -> Vec<u32> {
    let mut cues: Vec<u32> = audio.voices.keys().map(|cue| cue.0).collect();
    cues.sort_unstable();
    cues
}

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
    assert_eq!(cues(&audio), [1], "only the sound on device a plays on");
    assert_eq!(sounds(&mut mix_a), 1, "the sound of the other cues goes");
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
    assert_eq!(cues(&audio), [1]);
    assert_eq!(sounds(&mut mix_a), 1);
}

#[test]
fn opening_devices_sets_the_time_of_their_last_try_for_the_retry_to_wait() {
    let (mut audio, ..) = open_on_both(Fake::default());
    audio.tried = Instant::now()
        .checked_sub(2 * RETRY)
        .expect("the clock is past two retries");
    audio.patched(patch(&show(Channels::MAIN, Channels::Stereo(11, 12))));
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
        audio.tried = Instant::now()
            .checked_sub(2 * RETRY)
            .expect("the clock is past two retries");
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
    let fake = Fake {
        delay: Duration::from_millis(300),
        ..Fake::default()
    };
    let opened = Arc::clone(&fake.opened);
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(fake);
    let began = Instant::now();
    let workspace = show(Channels::MAIN, Channels::Stereo(0, 1));
    audio.sync(workspace, READY_AHEAD);
    // A retry that falls due while the first open runs starts no second
    audio.tried = Instant::now()
        .checked_sub(2 * RETRY)
        .expect("the clock is past two retries");
    audio.sync(show(Channels::MAIN, Channels::Stereo(0, 1)), READY_AHEAD);
    assert!(began.elapsed() < Duration::from_millis(250));
    assert_eq!(audio.problems(), [], "nothing is told while they open");
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
    assert_eq!(cues(&audio), [1]);
    // Pasting one back opens device b alone
    let back = cut.apply(Command::PasteOutputs {
        at: 1,
        outputs: vec![sub],
        restore: true,
    });
    change(&mut audio, &back);
    assert_eq!(opened.load(Ordering::SeqCst), 1, "only device b opens");
    assert!(is_open(&audio, "a"));
    assert_eq!(cues(&audio), [1]);
    assert_eq!(sounds(&mut mix_a), 1);
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
    audio
        .mixers
        .insert(Some("b".to_owned()), Err(fault.clone()));
    let sub = workspace.output(SUB).cloned().expect("the show has Sub");
    let copied = workspace.apply(Command::PasteOutputs {
        at: 2,
        outputs: vec![sub],
        restore: false,
    });
    audio.patched(patch(&copied));
    settle(&mut audio);
    assert_eq!(opened.load(Ordering::SeqCst), 0);
    assert!(matches!(
        audio.mixers.get(&Some("b".to_owned())),
        Some(Err(Fault::Unplayable(_)))
    ));
}
