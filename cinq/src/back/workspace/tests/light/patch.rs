// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the patch, its undo, and the addresses and levels of its fields

use super::*;
use crate::back::{PatchEdit, parse_address, parse_percent};

fn names(workspace: &Workspace) -> Vec<&str> {
    workspace
        .patch()
        .iter()
        .map(|instrument| instrument.name.as_str())
        .collect()
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
fn bringing_back_kept_changes_brings_back_the_patch() {
    let kept = lit(Vec::new());
    let restored = workspace(Vec::new()).apply(Command::Restore(
        kept.lists().to_vec(),
        Vec::new(),
        kept.patch().to_vec(),
    ));
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

#[test]
fn instrument_added_after_a_removal_does_not_take_the_level_of_the_removed() {
    let levelled = lit(vec![light(1, &[(2, 100.0)])])
        .apply(GO)
        .apply(tick(4.0))
        .apply(Command::Delete(vec![CueId(1)]));
    assert_eq!(level(&levelled, 2), 100.0);
    let added = levelled
        .apply(Command::Patch(PatchEdit::Remove(InstrumentId(2))))
        .apply(Command::Patch(PatchEdit::Add));
    assert_eq!(added.patch()[1].id, InstrumentId(3));
    assert_eq!(level(&added, 3), 0.0);
    assert_eq!(added.universes()[&1][..2], [0, 0]);
}

#[test]
fn instrument_is_not_added_after_the_last_address_of_the_last_universe() {
    let added = |address| {
        let edit = PatchEdit::Address(InstrumentId(2), 32_768, address);
        lit(Vec::new())
            .apply(Command::Patch(edit))
            .apply(Command::Patch(PatchEdit::Add))
    };
    assert_eq!(names(&added(512)), ["1", "2"]);
    assert_eq!(added(511).patch()[2].address_text(), "32768/512");
}

#[test]
fn a_light_cue_that_sets_no_instrument_or_names_one_the_patch_lacks_is_unlit() {
    let workspace = lit(vec![
        light(1, &[(1, 50.0)]),
        light(2, &[]),
        light(3, &[(1, 50.0), (99, 50.0)]),
        light(4, &[(98, 10.0), (99, 20.0)]),
    ]);
    let told = |workspace: &Workspace| -> Vec<(u32, String)> {
        workspace
            .unlit_cues()
            .map(|(cue, unlit)| (cue.id.0, unlit.to_string()))
            .collect()
    };
    assert_eq!(
        told(&workspace),
        [
            (2, "sets no instrument".to_owned()),
            (3, "names instrument 99, which the patch lacks".to_owned()),
            (
                4,
                "names instruments 98, 99, which the patch lacks".to_owned()
            ),
        ]
    );
    // Removing an instrument keeps its levels in the cues, so that undo restores them
    let removed = workspace.apply(Command::Patch(PatchEdit::Remove(InstrumentId(1))));
    assert_eq!(told(&removed).len(), 4);
    assert_eq!(removed.cue(CueId(1)).unwrap().lights.len(), 1);
    assert_eq!(told(&removed.apply(Command::Undo)).len(), 3);
}
