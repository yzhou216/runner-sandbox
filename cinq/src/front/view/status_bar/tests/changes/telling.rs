// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what the notice of a change tells and of when it shows

use super::*;

#[test]
fn a_change_to_a_cue_is_not_told_while_the_outputs_show_so_that_their_keys_stay() {
    let edited = rain().apply(Command::Edit(CueId(1), Edit::File("storm.ogg".to_owned())));
    let app = |workspace: Workspace, outputs_shown| App {
        workspace,
        outputs_shown,
        ..opened(false, false)
    };
    assert!(notice(&app(edited.clone(), false)).is_some());
    assert_eq!(notice(&app(edited, true)), None);
    // A change to the outputs is told there
    let renamed = rain().apply(rename(1, "Pit"));
    assert!(notice(&app(renamed, true)).is_some_and(|told| told.contains("Pit")));
}

#[test]
fn the_notice_of_a_delete_or_a_new_file_tells_that_the_cue_it_changed_stopped() {
    let app = |change: Command, stopped: &[CueId]| App {
        workspace: rain().apply(change),
        notices: Notices {
            stopped: stopped.to_vec(),
            ..Notices::default()
        },
        ..App::default()
    };
    let delete = || Command::Delete(vec![CueId(1)]);
    let file = || Command::Edit(CueId(1), Edit::File("hail.ogg".into()));
    assert_eq!(
        notice(&app(delete(), &[CueId(1)])).as_deref(),
        Some("Deleted cue 1, which stopped | u: undo")
    );
    assert_eq!(
        notice(&app(file(), &[CueId(1)])).as_deref(),
        Some("Changed the file of cue 1, which stopped | u: undo")
    );
    assert_eq!(
        notice(&app(file(), &[])).as_deref(),
        Some("Changed the file of cue 1 | u: undo")
    );
    // The undo of the change does not claim the stop
    let undone = App {
        workspace: rain().apply(file()).apply(Command::Undo),
        notices: Notices {
            stopped: vec![CueId(1)],
            ..Notices::default()
        },
        ..App::default()
    };
    assert_eq!(
        notice(&undone).as_deref(),
        Some("Undid: changed the file of cue 1 | Control+r: redo")
    );
}

#[test]
fn an_undo_or_redo_of_several_changes_tells_how_many_and_the_last() {
    let added = (0..5).fold(rain(), |workspace, _| {
        workspace.apply(Command::Add {
            list: 0,
            kind: Kind::Memo,
            at: crate::back::Place::End,
            count: 1,
        })
    });
    let app = |workspace, turns| App {
        workspace,
        notices: Notices {
            turns,
            ..Notices::default()
        },
        ..App::default()
    };
    let undone = (0..3).fold(added, |workspace, _| workspace.apply(Command::Undo));
    assert_eq!(
        notice(&app(undone.clone(), 3)).as_deref(),
        Some("Undid 3 changes, the last: added cue 4 | Control+r: redo")
    );
    assert_eq!(
        notice(&app(undone.clone(), 1)).as_deref(),
        Some("Undid: added cue 4 | Control+r: redo")
    );
    let redone = undone.apply(Command::Redo).apply(Command::Redo);
    assert_eq!(
        notice(&app(redone, 2)).as_deref(),
        Some("Redid 2 changes, the last: added cue 5 | u: undo")
    );
}

#[test]
fn the_notice_of_a_time_that_reads_as_no_time_tells_that_it_counts_as_zero() {
    let told = |time: &str| {
        notice(&App {
            workspace: rain().apply(Command::Edit(
                CueId(1),
                Edit::PreWait(time.to_owned().into()),
            )),
            ..App::default()
        })
    };
    assert_eq!(
        told("1:0:0:5").as_deref(),
        Some("Changed the pre-wait of cue 1 to zero | u: undo")
    );
    assert_eq!(
        told("2").as_deref(),
        Some("Changed the pre-wait of cue 1 | u: undo")
    );
}

#[test]
fn an_output_name_with_a_line_break_is_told_on_one_line() {
    let app = App {
        workspace: rain().apply(rename(1, "Side\nfill")),
        ..opened(false, false)
    };
    assert!(notice(&app).is_some_and(|told| told.contains("Side fill")));
}
