// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of undo and redo for outputs: what they select and where they leave
//! the view

use super::*;

#[test]
fn undo_and_redo_select_the_output_that_the_change_touched() {
    let cut = typed(on(3), "dd");
    let undone = typed(cut, "u");
    assert_eq!((names(&undone), at(&undone)), (ALL.to_vec(), 3));
    let redone = after(undone, [Message::Redo(1)]);
    assert_eq!((names(&redone), at(&redone)), (without(&[2]), 3));
    // Restored output is selected, wherever the keys left the selection
    let far = typed(typed(on(3), "dd"), "5j");
    assert_eq!(at(&typed(far, "u")), 3);
    // Move selects the output that moved, both ways
    let moved = after(on(3), [Message::Move(2)]);
    let back = after(moved, [Message::Undo(1)]);
    assert_eq!((names(&back), at(&back)), (ALL.to_vec(), 3));
    let again = after(back, [Message::Redo(1)]);
    assert_eq!(at(&again), 5);
    // Counts go that many changes back
    let two = after(
        on(1),
        [Message::Move(1), Message::Move(1), Message::Undo(2)],
    );
    assert_eq!(names(&two), ALL);
}

#[test]
fn undo_of_a_change_to_a_cue_leaves_the_outputs_for_the_cue_list_as_a_jump() {
    let shown = after(
        show(cue(2)),
        [Message::Add(Place::After, 1), Message::ShowOutputs],
    );
    assert!(shown.outputs_shown);
    let undone = typed(shown, "u");
    assert!(!undone.outputs_shown);
    assert_eq!(undone.workspace.cues().count(), 14);
    assert_eq!(place(&undone).0, 0);
    // Control+o returns to the outputs
    let back = after(undone, [Message::Back(1)]);
    assert!(back.outputs_shown);
    // Redo does the same for a change that only cues made
    let redone = after(back, [Message::Redo(1)]);
    assert!(!redone.outputs_shown);
    assert_eq!(redone.workspace.cues().count(), 15);
}

#[test]
fn undo_and_redo_in_show_mode_leave_the_outputs_as_they_are() {
    let app = after(locked(on(3)), [Message::Undo(1), Message::Redo(1)]);
    assert!(app.outputs_shown);
    assert_eq!(names(&app), ALL);
}

#[test]
fn undoing_an_addition_or_a_paste_selects_the_output_that_it_was_made_next_to() {
    let undone = |app: App, keys: &str| at(&typed(typed(app, keys), "u"));
    // Below and above, as `o u`, `O u`, `p u`, and `P u` of Vim end, with
    // counts, and at the first and the last place
    assert_eq!(undone(on(3), "o"), 3);
    assert_eq!(undone(on(3), "O"), 3);
    assert_eq!(undone(on(3), "3o"), 3);
    assert_eq!(undone(on(1), "O"), 1);
    assert_eq!(undone(on(10), "o"), 10);
    assert_eq!(undone(typed(on(3), "yy"), "p"), 3);
    assert_eq!(undone(typed(on(3), "yj"), "2P"), 3);
    // Wherever the selection went before the undo
    let moved = typed(typed(on(3), "o"), "4j");
    assert_eq!(at(&typed(moved, "u")), 3);
    // Redo selects what was added again, and undo goes back to the anchor
    let redone = after(typed(typed(on(3), "o"), "u"), [Message::Redo(1)]);
    assert_eq!(at(&redone), 4);
    assert_eq!(at(&typed(redone, "u")), 3);
    // Undo of a delete still selects what it brought back
    assert_eq!(at(&typed(typed(on(3), "dd"), "u")), 3);
}

#[test]
fn undo_and_redo_show_the_place_that_a_patch_or_output_change_touched() {
    // Light patch edit shows its tab, though the inspector was elsewhere
    let patched = after(
        show(cue(2)),
        [Message::Patch(PatchEdit::Add), Message::Tab(Tab::Basics)],
    );
    let undone = after(patched, [Message::Undo(1)]);
    assert_eq!(undone.tab, Tab::Patch);
    let redone = after(
        after(undone, [Message::Tab(Tab::Levels)]),
        [Message::Redo(1)],
    );
    assert_eq!(redone.tab, Tab::Patch);
    // Output edit made with the outputs in sight shows them again from the cue list
    let moved = after(on(3), [Message::Move(2)]).showing_cues();
    assert!(!moved.outputs_shown);
    let undone = after(moved, [Message::Undo(1)]);
    assert!(undone.outputs_shown);
    assert_eq!((names(&undone), at(&undone)), (ALL.to_vec(), 3));
}

#[test]
fn a_count_that_reaches_a_change_to_the_outputs_shows_them_as_one_change_does() {
    let made = after(
        show(cue(2)),
        [Message::ShowOutputs, Message::Add(Place::After, 1)],
    )
    .showing_cues();
    let made = after(made, [Message::Add(Place::After, 1)]);
    let undone = after(made, [Message::Undo(2)]);
    assert!(undone.outputs_shown);
    assert_eq!(undone.workspace.cues().count(), 14);
    assert_eq!(undone.workspace.outputs().len(), 1);
    assert_eq!(undone.output, Some(undone.workspace.outputs()[0].id));
    // Control+o leaves them for the cue list, where the count began
    let back = after(undone, [Message::Back(1)]);
    assert!(!back.outputs_shown);
    // Redo does the same on its way, and ends where the last change lies
    let redone = after(back, [Message::Redo(2)]);
    assert!(!redone.outputs_shown);
    assert_eq!(redone.workspace.cues().count(), 15);
    assert_eq!(redone.workspace.outputs().len(), 2);
    assert_eq!(redone.notices.turns, 2);
}
