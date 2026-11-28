// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which outputs that cannot play the Levels tab counts, and of how
//! it names them

use super::{cut, labels, named, silent, widest};
use crate::{
    back::{Command, Cue, CueId, Fault, Kind, OutputEdit, OutputId, Sends, fresh},
    front::{App, Problems, fixtures::app_of},
};

#[test]
fn only_outputs_that_the_cue_sends_to_count_as_unable_to_play() {
    let sends: Sends = [(OutputId(1), 0.0)].into_iter().collect();
    let cue = Cue {
        sends: Some(sends.clone()),
        ..Cue::new(CueId(1), "1", Kind::Audio, "Music").build()
    };
    let workspace = fresh(vec![cue]).apply(Command::add_output(1, 1));
    let missing = || Fault::Missing("there is no audio device".to_owned());
    let app = App {
        workspace,
        problems: Problems {
            audio: vec![(OutputId(1), missing()), (OutputId(2), missing())],
            ..Problems::default()
        },
        ..App::default()
    };
    assert_eq!(silent(&app, &sends), [&missing()]);
}

#[test]
fn an_output_that_cannot_play_is_named_with_what_is_wrong_with_it() {
    assert_eq!(named("House L", 20, None), "House L");
    let faults = [
        (Fault::Missing("gone".to_owned()), "House L (not found)"),
        (Fault::Refused("busy".to_owned()), "House L (cannot open)"),
        (Fault::Unplayable("no".to_owned()), "House L (cannot open)"),
        (Fault::Stopped("pulled".to_owned()), "House L (stopped)"),
        (Fault::Unanswered, "House L (not answering)"),
        (Fault::Channels(2), "House L (past its channels)"),
    ];
    faults
        .iter()
        .for_each(|(fault, words)| assert_eq!(named("House L", 20, Some(fault)), *words));
}

#[test]
fn a_long_name_is_cut_at_its_start_and_its_fault_is_not() {
    let long = "Missing speaker box number five";
    assert_eq!(named(long, 20, None), "\u{2026}ker box number five");
    assert_eq!(
        named(long, 20, Some(&Fault::Unanswered)),
        "\u{2026}ker box number five (not answering)"
    );
}

#[test]
fn names_that_share_a_beginning_stay_apart_when_cut() {
    let cut = |name| cut(name, 20);
    assert_ne!(cut("House speaker pair 01"), cut("House speaker pair 02"));
    assert_eq!(cut("House speaker pair 16"), "\u{2026}use speaker pair 16");
}

#[test]
fn the_room_of_the_names_counts_the_columns_that_a_wide_character_takes() {
    let names = |names: &[&str]| {
        widest(
            names
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
        )
    };
    assert_eq!(names(&["House L", "Sub"]), 7);
    assert_eq!(names(&["\u{4e3b}\u{97f3}\u{54cd}"]), 6);
    assert_eq!(names(&[]), 0);
}

#[test]
fn a_fault_on_one_output_leaves_the_room_of_the_other_names_as_it_was() {
    let workspace = fresh(Vec::new()).apply(Command::add_output(1, 2));
    let plain = app_of(workspace.clone());
    let faulty = App {
        workspace,
        problems: Problems {
            audio: vec![(OutputId(2), Fault::Missing("gone".to_owned()))],
            ..Problems::default()
        },
        ..App::default()
    };
    let (room, names) = labels(&plain);
    assert_eq!(labels(&faulty).0, room);
    assert!(names.iter().all(|(_, fault)| fault.is_none()));
    let (_, told) = labels(&faulty);
    assert_eq!(told[1].0, "Output 1 (not found)");
    assert!(told[1].1.is_some() && told[0].1.is_none() && told[2].1.is_none());
}

#[test]
fn an_output_name_with_a_line_break_is_told_on_one_line() {
    let app = app_of(fresh(Vec::new()).apply(Command::EditOutput(
        OutputId(1),
        OutputEdit::Name("House\r\nL".to_owned()),
    )));
    assert_eq!(labels(&app).1[0].0, "House L");
}
