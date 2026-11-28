// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the State column for a cue that plays, waits, or fails to load

use iced::widget::text;

use super::{
    super::columns::states::{Look, lines, state},
    show, told, yellow,
};
use crate::{
    back::{Command, Cue, CueId, InstrumentId, Kind, OutputId, Sends, Workspace, fresh},
    front::view::theme,
};

fn shown(workspace: &Workspace) -> (String, bool) {
    let list = &workspace.lists()[0];
    let cue = &list.cues[0];
    let (state, style) = state(workspace, cue, None, false);
    (state, yellow(style))
}

#[test]
fn a_cue_that_sends_to_no_output_left_reads_in_yellow_but_one_that_sends_somewhere_does_not() {
    let sends = [(OutputId(2), 0.0)].into();
    assert_eq!(shown(&show(Some(sends))), ("NO OUTPUT".to_owned(), true));
    // Somewhere, or with no sends named, reads as any other cue
    let both: Sends = [(OutputId(1), 0.0), (OutputId(2), 0.0)].into();
    assert_eq!(shown(&show(Some(both))).0, "");
    assert_eq!(shown(&show(None)).0, "");
}

/// Returns the lines of the State column of the only cue, which is standby
fn standby_lines(workspace: &Workspace) -> Vec<String> {
    let cue = &workspace.lists()[0].cues[0];
    told(lines(workspace, cue, Look::default().with_next(true)))
}

#[test]
fn a_standby_cue_reads_next_and_then_its_problem_if_it_has_one() {
    let workspace = show(None);
    let cue = &workspace.lists()[0].cues[0];
    assert_eq!(standby_lines(&workspace), ["NEXT"]);
    assert!(lines(&workspace, cue, Look::default()).is_empty());
    let stranded = show(Some([(OutputId(2), 0.0)].into()));
    assert_eq!(standby_lines(&stranded), ["NEXT", "NO OUTPUT"]);
    let unloaded = fresh(vec![Cue::new(CueId(1), "1", Kind::Audio, "Rain").build()]);
    assert_eq!(standby_lines(&unloaded), ["NEXT", "NO FILE"]);
}

#[test]
fn a_cue_that_a_panic_is_fading_reads_fading_in_the_color_of_a_running_cue() {
    let read = |workspace: &Workspace| state(workspace, &workspace.lists()[0].cues[0], None, false);
    let running = show(None)
        .apply(Command::Go { list: 0 })
        .apply(Command::Tick(1.0));
    assert!(read(&running).0.ends_with("left"));
    let panicked = running.apply(Command::Panic).apply(Command::Tick(0.1));
    let (text_of, style) = read(&panicked);
    let theme = theme::modus_vivendi();
    assert_eq!(text_of, "fading");
    assert_eq!(style(&theme).color, text::success(&theme).color);
}

#[test]
fn a_light_cue_that_sets_no_instrument_or_names_one_the_patch_lacks_reads_it_in_yellow() {
    let light = |levels: &[(u32, f32)]| {
        let lights = levels
            .iter()
            .map(|&(id, level)| (InstrumentId(id), level))
            .collect();
        let cue = Cue::new(CueId(1), "1", Kind::Light, "Light")
            .lights(lights)
            .build();
        fresh(vec![cue])
    };
    assert_eq!(shown(&light(&[])), ("NO LEVELS".to_owned(), true));
    assert_eq!(shown(&light(&[(99, 50.0)])), ("UNPATCHED".to_owned(), true));
}

#[test]
fn a_standby_cue_that_runs_keeps_next_above_its_state() {
    let running = show(None)
        .apply(Command::Go { list: 0 })
        .apply(Command::Tick(1.0));
    let lines = standby_lines(&running);
    assert_eq!((lines.len(), lines[0].as_str()), (2, "NEXT"));
    assert!(lines[1].ends_with("left"), "{lines:?}");
}
