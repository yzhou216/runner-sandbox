// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the sounds that play while the outputs change, on mixers without
//! devices

use tempfile::TempDir;

use super::{
    fakes::{Fake, MAIN, SUB, named, sounds},
    voice::{audio, fired_promising},
    waiting,
};
use crate::back::{
    Audio, Channels, Command, CueId, Device, Edit, OutputEdit, OutputId, READY_AHEAD, Sends,
    Workspace,
    audio::sound::{
        mixer::{Mix, Mixer},
        opener::Opener,
    },
    workspace::tests::GO,
};

/// Returns audio with the devices a and b open, the mixes of their mixers,
/// and the show in which the sound of one cue plays on both, from Main on a
/// and Sub on b
fn on_both() -> (Audio, Workspace, Mix, Mix, TempDir) {
    let (directory, workspace) = fired_promising(audio(), 2.0, 2.0);
    let workspace = workspace
        .apply(Command::EditOutput(MAIN, OutputEdit::Device(named("a"))))
        .apply(Command::add_output(1, 1))
        .apply(Command::EditOutput(SUB, OutputEdit::Device(named("b"))));
    let workspace = sent_to(workspace, &[MAIN, SUB]);
    let mut audio = Audio::open(None);
    audio.opener = Opener::with(Fake::default());
    let (a, mix_a) = Mixer::bench(2, 2, 8000, 16);
    let (b, mix_b) = Mixer::bench(2, 2, 8000, 16);
    audio.mixers.insert(Some("a".to_owned()), Ok(a));
    audio.mixers.insert(Some("b".to_owned()), Ok(b));
    (audio, workspace, mix_a, mix_b, directory)
}

/// Returns `workspace` with the sends of cue 1 changed to `outputs`, each at
/// no change of level
fn sent_to(workspace: Workspace, outputs: &[OutputId]) -> Workspace {
    let sends: Sends = outputs.iter().map(|&output| (output, 0.0)).collect();
    workspace.apply(Command::Edit(CueId(1), Edit::Sends(Some(sends))))
}

/// Returns how many sounds `mix` holds after one frame, in which a sound
/// that was cut and begun again would still have its old sound fading out
/// beside it, for ten milliseconds
fn after_a_frame(mix: &mut Mix) -> usize {
    mix.render(&mut [0.0_f32; 2]);
    mix.sounds()
}

/// Syncs `audio` with `workspace`, which standby puts back on cue 1, until
/// the sound for the next GO of the cue is made ready, and returns the
/// workspace
fn readied(audio: &mut Audio, workspace: Workspace) -> Workspace {
    let mut workspace = workspace.apply(Command::Standby {
        list: 0,
        cue: Some(CueId(1)),
    });
    waiting(|| {
        workspace = audio.sync(std::mem::take(&mut workspace), READY_AHEAD);
        audio.preloader.ready(CueId(1)).then_some(())
    })
    .expect("the sound is made ready");
    workspace
}

/// Main moving to channels that its device was not opened for, so that the
/// device opens again
fn wider() -> Command {
    Command::EditOutput(MAIN, OutputEdit::Channels(Channels::Stereo(11, 12)))
}

#[test]
fn moving_an_output_leaves_the_sound_on_two_devices_playing() {
    let (mut audio, workspace, mut mix_a, mut mix_b, _directory) = on_both();
    let workspace = audio.sync(workspace, READY_AHEAD);
    assert_eq!((sounds(&mut mix_a), sounds(&mut mix_b)), (1, 1));
    let moved = Command::MoveOutput {
        output: SUB,
        by: -1,
    };
    audio.sync(workspace.apply(moved), READY_AHEAD);
    assert_eq!(
        (after_a_frame(&mut mix_a), after_a_frame(&mut mix_b)),
        (1, 1)
    );
}

#[test]
fn a_sound_that_starts_on_a_device_plays_on_from_where_the_run_is_and_keeps_the_sound_for_its_next_go()
 {
    let (mut audio, workspace, _mix_a, _mix_b, _directory) = on_both();
    let workspace = audio.sync(workspace, READY_AHEAD);
    // The run has played a second, and standby is back on its cue
    let workspace = readied(&mut audio, workspace.apply(Command::Tick(1.0)));
    // A send to one device only ends the sound on the other, and a send
    // to both again starts it on the other alone
    let workspace = audio.sync(sent_to(workspace, &[MAIN]), READY_AHEAD);
    // The sound that stays reports a start of its file, as no device plays
    // it, which pulls the clock back, so the run is read before the sync.
    // Each sync above pulled the clock back, by as much as the waits took
    // syncs, so a second more makes the place of the run certain
    let both = sent_to(workspace, &[MAIN, SUB]).apply(Command::Tick(1.0));
    let played = both.active_of(CueId(1)).expect("the cue runs").played();
    audio.sync(both, READY_AHEAD);
    let at = audio.voices[&CueId(1)].position().expect("the voice plays");
    assert!(
        played > 0.5 && (at - played).abs() < 0.05,
        "{at} of {played}"
    );
    assert!(audio.preloader.ready(CueId(1)));
}

#[test]
fn choosing_another_default_device_leaves_the_sounds_on_named_devices_playing() {
    let (mut audio, workspace, _mix_a, mut mix_b, _directory) = on_both();
    audio.default = Some("a".to_owned());
    let on_default = workspace.apply(Command::EditOutput(
        MAIN,
        OutputEdit::Device(Device::Default),
    ));
    let workspace = audio.sync(on_default, READY_AHEAD);
    assert_eq!(sounds(&mut mix_b), 1);
    audio.route(Some("c"));
    audio.sync(workspace, READY_AHEAD);
    assert!(matches!(
        audio.mixers.get(&Some("b".to_owned())),
        Some(Ok(_))
    ));
    assert_eq!(after_a_frame(&mut mix_b), 1);
    assert!(!audio.mixers.contains_key(&Some("a".to_owned())));
}

#[test]
fn a_device_that_reopens_leaves_the_sound_on_the_other_playing() {
    let (mut audio, workspace, _mix_a, mut mix_b, _directory) = on_both();
    let workspace = audio.sync(workspace, READY_AHEAD);
    audio.sync(workspace.apply(wider()), READY_AHEAD);
    assert_eq!(after_a_frame(&mut mix_b), 1);
}

#[test]
fn a_run_whose_only_device_reopens_keeps_the_sound_made_ready_for_its_next_go() {
    let (mut audio, workspace, _mix_a, _mix_b, _directory) = on_both();
    let workspace = audio
        .sync(sent_to(workspace, &[MAIN]), READY_AHEAD)
        .apply(Command::Tick(1.0));
    let workspace = readied(&mut audio, workspace);
    // The device opens again, which ends the only sound of the run, and the
    // sound that starts in its place opens the file where the run is
    audio.sync(workspace.apply(wider()), READY_AHEAD);
    assert!(audio.preloader.ready(CueId(1)));
}

#[test]
fn a_send_that_returns_starts_the_sound_on_its_device_alone() {
    let (mut audio, workspace, mut mix_a, mut mix_b, _directory) = on_both();
    let workspace = audio.sync(workspace, READY_AHEAD);
    let workspace = audio.sync(sent_to(workspace, &[MAIN]), READY_AHEAD);
    assert_eq!((sounds(&mut mix_a), sounds(&mut mix_b)), (1, 0));
    audio.sync(sent_to(workspace, &[MAIN, SUB]), READY_AHEAD);
    assert_eq!(
        (after_a_frame(&mut mix_a), after_a_frame(&mut mix_b)),
        (1, 1)
    );
}

#[test]
fn a_run_that_the_clock_started_long_before_its_first_tick_starts_every_device_together() {
    let (mut audio, workspace, _mix_a, _mix_b, _directory) = on_both();
    let workspace = audio.sync(workspace, READY_AHEAD);
    let workspace = readied(&mut audio, workspace);
    // GO fires the cue again, and a long frame passes before the next sync
    let workspace = audio.sync(workspace.apply(GO).apply(Command::Tick(0.5)), READY_AHEAD);
    let at = audio.voices[&CueId(1)].positions();
    // Both devices start at the start of the clip, the first with the sound
    // made ready and the second from its file, so that they play together
    assert!(
        workspace
            .active_of(CueId(1))
            .expect("the cue runs")
            .played()
            > 0.4
            && at.len() == 2
            && at.iter().all(|place| *place < 0.1),
        "{at:?}"
    );
}

#[test]
fn a_run_that_starts_with_no_device_open_keeps_the_sound_made_ready() {
    let (mut audio, workspace, _mix_a, _mix_b, _directory) = on_both();
    let workspace = audio.sync(workspace, READY_AHEAD);
    let workspace = readied(&mut audio, workspace);
    // No device is open when GO fires the cue, so no sound starts yet, and
    // the sound made ready waits for the device that comes to play it
    audio.mixers.clear();
    audio.sync(workspace.apply(GO), READY_AHEAD);
    assert!(audio.preloader.ready(CueId(1)));
}
