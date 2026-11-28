// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the commands that play, pause, and stop cues over OSC

use super::*;
use crate::back::Media;

#[test]
fn go_fires_the_standby_of_the_list_in_view_as_the_key_does() {
    let by_key = after(playable(None), [Message::Go]);
    let by_osc = remote(playable(None), Remote::Go);
    assert_eq!(running(&by_osc), [1]);
    assert_eq!(running(&by_osc), running(&by_key));
    assert_eq!(by_osc.workspace.lists()[0].standby, Some(CueId(2)));
    let preshow = after(playable(None), [Message::SwitchList(1)]);
    assert_eq!(running(&remote(preshow, Remote::Go)), [20]);
}

#[test]
fn go_and_panic_act_in_show_mode_and_close_the_help_page() {
    let app = helping(locked(playable(None)));
    let gone = remote(app, Remote::Go);
    assert_eq!(running(&gone), [1]);
    assert!(!gone.help);
    let app = App { help: true, ..gone };
    let panicked = remote(app, Remote::Panic);
    assert!(!panicked.help);
    let by_key = after(
        helping(after(playable(None), [Message::Go])),
        [Message::Panic],
    );
    assert_eq!(
        format!("{:?}", panicked.workspace.active()),
        format!("{:?}", by_key.workspace.active())
    );
}

#[test]
fn every_command_leaves_hint_mode() {
    let commands = [
        Remote::Go,
        Remote::Panic,
        Remote::Pause,
        Remote::Resume,
        Remote::Start("1".into()),
        Remote::Stop("1".into()),
        Remote::PauseCue("1".into()),
        Remote::ResumeCue("1".into()),
        Remote::Playhead("3".into()),
    ];
    for command in commands {
        let (app, _) = playable(None).hinting();
        assert!(app.hints.is_some());
        assert!(remote(app, command.clone()).hints.is_none(), "{command:?}");
    }
}

#[test]
fn start_fires_the_cue_of_that_number_and_leaves_standby_alone() {
    let app = remote(playable(None), Remote::Start("21".into()));
    assert_eq!(running(&app), [21]);
    assert_eq!(app.workspace.lists()[1].standby, Some(CueId(20)));
    assert_eq!(app.workspace.lists()[0].standby, Some(CueId(1)));
}

#[test]
fn pause_cue_resume_cue_and_stop_act_on_the_cue_of_that_number() {
    let app = after(playable(None), [Message::Go, Message::Go]);
    let held = remote(app, Remote::PauseCue("2".into()));
    let paused = |app: &App, id: u32| app.workspace.active_of(CueId(id)).unwrap().paused;
    assert!(paused(&held, 2));
    assert!(!paused(&held, 1));
    let freed = remote(held, Remote::ResumeCue("2".into()));
    assert!(!paused(&freed, 2));
    let stopped = remote(freed, Remote::Stop("1".into()));
    assert_eq!(running(&stopped), [2]);
}

#[test]
fn pause_and_resume_act_on_every_running_cue() {
    let app = after(playable(None), [Message::Go, Message::Go]);
    let held = remote(app, Remote::Pause);
    assert!(held.workspace.active().iter().all(|run| run.paused));
    let freed = remote(held, Remote::Resume);
    assert!(freed.workspace.active().iter().all(|run| !run.paused));
}

#[test]
fn start_of_a_cue_that_cannot_play_does_nothing_and_says_why_but_a_loading_cue_starts() {
    let broken = || {
        let app = playable(None);
        let file = app.workspace.cue(CueId(21)).unwrap().file.clone();
        app.send(loaded(21, &file, Media::Failed("no such file".to_owned())))
    };
    let told = remote(broken(), Remote::Start("21".into()));
    assert_eq!(
        told.problems.osc_ignored.as_deref(),
        Some("OSC from 192.168.1.5: cue 21 cannot play: no such file")
    );
    assert!(running(&told).is_empty());
    // Moving standby to such a cue is not a command that fails
    let moved = remote(broken(), Remote::Playhead("21".into()));
    assert!(moved.problems.osc_ignored.is_none());
    // A cue that only waits to load is no problem
    let loading = remote(show(None), Remote::Start("1".into()));
    assert!(loading.problems.osc_ignored.is_none());
}
