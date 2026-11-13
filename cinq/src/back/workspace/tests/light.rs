// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of lights: the patch and its undo, light cues, which fade the
//! instruments that they name and leave the rest, and the DMX that the
//! levels make

use std::path::Path;

use super::*;
use crate::back::{Instrument, InstrumentId, PatchEdit, parse_address, parse_percent};

/// Returns a light cue that fades instruments to `levels` over four seconds
fn light(id: u32, levels: &[(u32, f32)]) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Light, "Light")
        .duration(4.0)
        .lights(
            levels
                .iter()
                .map(|&(instrument, level)| (InstrumentId(instrument), level))
                .collect(),
        )
        .build()
}

/// Returns a workspace of `cues` with instruments 1 and 2 at addresses 1
/// and 2
fn lit(cues: Vec<Cue>) -> Workspace {
    let instrument = |id, address| Instrument {
        id: InstrumentId(id),
        name: id.to_string(),
        universe: 1,
        address,
    };
    workspace(cues).with_patch(vec![instrument(1, 1), instrument(2, 2)])
}

fn level(workspace: &Workspace, instrument: u32) -> f32 {
    workspace.light_level(InstrumentId(instrument))
}

fn tick(seconds: f64) -> Command {
    Command::Tick(seconds)
}

fn names(workspace: &Workspace) -> Vec<&str> {
    workspace
        .patch()
        .iter()
        .map(|instrument| instrument.name.as_str())
        .collect()
}

#[test]
fn light_cue_fades_its_instruments_and_leaves_them_where_it_ends() {
    let fading = lit(vec![light(1, &[(1, 80.0)])]).apply(GO).apply(tick(1.0));
    assert_eq!(level(&fading, 1), 20.0);
    let done = fading.apply(tick(3.0));
    assert!(done.active().is_empty());
    assert_eq!(level(&done, 1), 80.0);
    assert_eq!(level(&done, 2), 0.0);
    assert_eq!(done.universes()[&1][..3], [204, 0, 0]);
}

#[test]
fn newer_light_cue_takes_over_only_the_instruments_it_names() {
    let workspace = lit(vec![
        light(1, &[(1, 100.0), (2, 100.0)]),
        light(2, &[(1, 0.0)]),
    ])
    .apply(GO)
    .apply(tick(2.0))
    .apply(GO)
    .apply(tick(2.0));
    assert_eq!(level(&workspace, 1), 25.0);
    assert_eq!(level(&workspace, 2), 100.0);
    let done = workspace.apply(tick(2.0));
    assert_eq!(level(&done, 1), 0.0);
}

#[test]
fn paused_or_stopped_light_cue_leaves_its_instruments_where_they_are() {
    let fading = lit(vec![light(1, &[(1, 80.0)])]).apply(GO).apply(tick(2.0));
    let paused = fading.clone().apply(Command::Pause(true)).apply(tick(1.0));
    assert_eq!(level(&paused, 1), 40.0);
    let resumed = paused.apply(Command::Pause(false)).apply(tick(1.0));
    assert_eq!(level(&resumed, 1), 60.0);
    let panicked = fading.apply(Command::Panic).apply(tick(1.0));
    assert!(panicked.active().is_empty());
    assert_eq!(level(&panicked, 1), 40.0);
}

#[test]
fn instruments_on_one_address_set_it_to_the_highest_of_their_levels() {
    let shared = Command::Patch(PatchEdit::Address(InstrumentId(2), 1, 1));
    let workspace = lit(vec![light(1, &[(1, 20.0), (2, 100.0)])])
        .apply(shared)
        .apply(GO)
        .apply(tick(4.0));
    assert_eq!(workspace.universes()[&1][..2], [255, 0]);
}

#[test]
fn added_instruments_follow_the_last_and_take_no_identity_that_a_cue_names() {
    let workspace = lit(vec![light(1, &[(7, 50.0)])])
        .apply(Command::Patch(PatchEdit::Address(InstrumentId(2), 3, 512)))
        .apply(Command::Patch(PatchEdit::Add));
    let added = &workspace.patch()[2];
    assert_eq!(added.id, InstrumentId(8));
    assert_eq!((added.universe, added.address), (4, 1));
    assert_eq!(added.address_text(), "4/1");
}

#[test]
fn edits_of_one_setting_of_the_patch_in_a_row_undo_as_one_change() {
    let rename = |name: &str| Command::Patch(PatchEdit::Name(InstrumentId(1), name.to_owned()));
    let renamed = lit(Vec::new()).apply(rename("F")).apply(rename("Front"));
    assert_eq!(names(&renamed), ["Front", "2"]);
    let undone = renamed.apply(Command::Undo);
    assert_eq!(names(&undone), ["1", "2"]);
    let removed = undone.apply(Command::Patch(PatchEdit::Remove(InstrumentId(1))));
    assert_eq!(names(&removed), ["2"]);
    assert_eq!(names(&removed.apply(Command::Undo)), ["1", "2"]);
}

#[test]
fn edits_of_the_levels_of_two_instruments_undo_one_at_a_time() {
    let set =
        |instrument, level| Command::Edit(CueId(1), Edit::Light(InstrumentId(instrument), level));
    let edited = lit(vec![light(1, &[])])
        .apply(set(1, Some(50.0)))
        .apply(set(2, Some(70.0)))
        .apply(set(2, Some(75.0)));
    let lights = |workspace: &Workspace| workspace.cue(CueId(1)).unwrap().lights.clone();
    assert_eq!(lights(&edited).len(), 2);
    let undone = edited.apply(Command::Undo);
    assert_eq!(lights(&undone).len(), 1);
    assert!(lights(&undone.apply(Command::Undo)).is_empty());
}

#[test]
fn show_file_holds_the_patch_and_the_levels_of_light_cues() {
    let show = lit(vec![light(1, &[(2, 50.0)])])
        .apply(Command::Patch(PatchEdit::Address(InstrumentId(2), 2, 12)))
        .show(Some(Path::new("/show")));
    assert_eq!(
        show,
        r#"[[instrument]]
id = 1
name = "1"
address = 1

[[instrument]]
id = 2
name = "2"
universe = 2
address = 12

[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "light"
name = "Light"
duration = 4.0

[list.cue.lights]
2 = 50.0
"#
    );
    let read = Workspace::from_show(&show, "/show").unwrap();
    assert_eq!(read.show(Some(Path::new("/show"))), show);
}

#[test]
fn show_file_with_an_address_or_a_light_level_out_of_range_cannot_be_read() {
    let instrument = |address| format!("[[instrument]]\nid = 1\naddress = {address}\n");
    assert!(Workspace::from_show(&instrument(512), "/show").is_ok());
    assert!(Workspace::from_show(&instrument(0), "/show").is_err());
    let cue = |level| {
        format!(
            "[[list]]\nname = \"Main\"\n[[list.cue]]\nid = 1\nkind = \"light\"\nlights = {{ 1 = {level} }}\n"
        )
    };
    assert!(Workspace::from_show(&cue(100.0), "/show").is_ok());
    assert!(Workspace::from_show(&cue(100.5), "/show").is_err());
}

#[test]
fn bringing_back_kept_changes_brings_back_the_patch() {
    let kept = lit(Vec::new()).contents();
    let restored = workspace(Vec::new()).apply(Command::Restore(kept));
    assert_eq!(names(&restored), ["1", "2"]);
    assert!(restored.apply(Command::Undo).patch().is_empty());
}

#[test]
fn addresses_and_light_levels_read_as_the_fields_show_them() {
    let instrument = Instrument {
        id: InstrumentId(1),
        name: String::new(),
        universe: 2,
        address: 101,
    };
    assert_eq!(parse_address(&instrument.address_text()), Some((2, 101)));
    assert_eq!(parse_address(" 12 "), Some((1, 12)));
    assert_eq!(parse_address("513"), None);
    assert_eq!(parse_address("0/1"), None);
    assert_eq!(parse_address("32769/1"), None);
    assert_eq!(instrument.to_string(), "1");
    assert_eq!(parse_percent("80%"), Some(Some(80.0)));
    assert_eq!(parse_percent(" "), Some(None));
    assert_eq!(parse_percent("101"), None);
}
