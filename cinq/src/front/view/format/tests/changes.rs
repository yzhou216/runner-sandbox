// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how a change to a show is told

use super::*;

#[test]
fn copied_and_cut_cues_are_told_by_the_number_of_the_cue_or_else_by_how_many() {
    assert_eq!(clipped("Copied", &[memo(4)]), "Copied cue 4");
    assert_eq!(clipped("Cut", &[memo(1), memo(2), memo(3)]), "Cut 3 cues");
}

#[test]
fn changed_cues_are_told_by_the_number_of_the_one_or_else_by_how_many() {
    let shown = fresh(vec![memo(1), memo(2), memo(3)]);
    let after = |command| told(&shown.clone().apply(command));
    let add = |count| Command::Add {
        list: 0,
        kind: Kind::Memo,
        at: Place::End,
        count,
    };
    let paste = |count| Command::Paste {
        list: 0,
        cues: vec![memo(1); count],
        at: Place::End,
    };
    let ids = |ids: [u32; 2]| ids.map(CueId).to_vec();
    assert_eq!(after(add(1)), "added cue 4");
    assert_eq!(after(add(3)), "added 3 cues");
    assert_eq!(after(paste(1)), "pasted cue 4");
    assert_eq!(after(paste(3)), "pasted 3 cues");
    assert_eq!(after(Command::Cut(vec![CueId(2)])), "cut cue 2");
    assert_eq!(after(Command::Cut(ids([1, 2]))), "cut 2 cues");
    assert_eq!(after(Command::Delete(vec![CueId(2)])), "deleted cue 2");
    assert_eq!(after(Command::Delete(ids([1, 2]))), "deleted 2 cues");
}

#[test]
fn loop_edits_are_told_by_the_part_of_the_loop_they_change() {
    let rain = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .looping(true)
        .build();
    let workspace = fresh(vec![rain]);
    let after = |edit| told(&workspace.clone().apply(Command::Edit(CueId(1), edit)));
    assert_eq!(
        after(Edit::LoopStart(2.0.into())),
        "changed the loop start of cue 1"
    );
    assert_eq!(
        after(Edit::LoopEnd(6.0.into())),
        "changed the loop end of cue 1"
    );
}

#[test]
fn a_file_typed_into_an_unnamed_cue_is_told_by_a_short_notice() {
    let workspace = fresh(vec![memo(1)]).apply(Command::Edit(
        CueId(1),
        Edit::FileAndName {
            file: "knock.ogg".into(),
            name: "knock".into(),
        },
    ));
    assert_eq!(told(&workspace), "changed the file of cue 1");
}

#[test]
fn a_move_is_told_to_enter_or_leave_a_group_only_where_the_depth_changes() {
    let within = |cue: Cue| Cue { depth: 1, ..cue };
    let group = Cue::new(CueId(2), "2", Kind::Group, "Group").build();
    let shown = fresh(vec![
        memo(1),
        group,
        within(memo(3)),
        within(memo(4)),
        memo(5),
    ]);
    let moved = |workspace: &Workspace, cue: u32, by: isize| {
        workspace.clone().apply(Command::Move {
            cue: CueId(cue),
            by,
        })
    };
    let into = moved(&shown, 1, 1);
    assert_eq!(told(&into), "moved cue 1 into a group");
    assert_eq!(told(&moved(&shown, 4, 1)), "moved cue 4 out of a group");
    assert_eq!(told(&moved(&shown, 3, 1)), "moved cue 3");
    let undone = into.apply(Command::Undo);
    let latest = undone.redoable().expect("the move can be redone");
    assert_eq!(change(&undone, latest), "moved cue 1 into a group");
}

#[test]
fn clearing_the_last_send_of_a_cue_is_told_as_playing_nowhere() {
    let rain = Cue::new(CueId(1), "1", Kind::Audio, "Rain").build();
    let workspace = fresh(vec![rain]);
    let after = |sends| {
        told(
            &workspace
                .clone()
                .apply(Command::Edit(CueId(1), Edit::Sends(sends))),
        )
    };
    assert_eq!(after(Some(Sends::new())), "changed cue 1 to play nowhere");
    assert_eq!(
        after(Some([(OutputId(1), -3.0)].into())),
        "changed where cue 1 plays"
    );
}

#[test]
fn a_cue_added_below_a_group_or_one_of_its_cues_is_told_to_be_in_a_group() {
    let group = Cue::new(CueId(1), "1", Kind::Group, "Group").build();
    let held = Cue::new(CueId(2), "2", Kind::Memo, "Note").depth(1).build();
    let after = |cues: Vec<Cue>, at: CueId, count| {
        told(&fresh(cues).apply(Command::Add {
            list: 0,
            kind: Kind::Memo,
            at: Place::After(at),
            count,
        }))
    };
    assert_eq!(
        after(vec![group.clone()], CueId(1), 1),
        "added cue 2 in a group"
    );
    assert_eq!(
        after(vec![group.clone(), held.clone()], CueId(2), 2),
        "added 2 cues in a group"
    );
    assert_eq!(
        after(vec![group, held, memo(3)], CueId(3), 1),
        "added cue 4"
    );
}

#[test]
fn an_edit_of_the_light_patch_is_told_by_the_instrument_it_changed() {
    let patched = |edits: &[PatchEdit]| {
        told(
            &edits
                .iter()
                .cloned()
                .fold(fresh(vec![memo(1)]), |shown, edit| {
                    shown.apply(Command::Patch(edit))
                }),
        )
    };
    let id = InstrumentId(1);
    assert_eq!(patched(&[PatchEdit::Add]), "added instrument 1");
    assert_eq!(
        patched(&[PatchEdit::Add, PatchEdit::Remove(id)]),
        "removed instrument 1"
    );
    assert_eq!(
        patched(&[PatchEdit::Add, PatchEdit::Name(id, "Front".to_owned())]),
        "renamed instrument 1 to Front"
    );
    assert_eq!(
        patched(&[PatchEdit::Add, PatchEdit::Address(id, 2, 12)]),
        "changed the address of instrument 1 to 2/12"
    );
}
