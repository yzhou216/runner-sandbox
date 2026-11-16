// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that edit cues editing the outputs while they show: the
//! register of outputs, the identity that a cut output takes back, the
//! notices, undo and redo, and the keys that enter the name

use iced::keyboard::{Key, Modifiers};

use super::{
    chord,
    history::{after, cue, place},
    output_rows::{at, outputs},
    paging::letter,
    press, show,
};
use crate::{
    back::{CueId, Edit, OutputId, Place, Sends},
    front::{
        App, Caret, Focus, Message, Setting, keymap::Action, update::clipboard::Register,
        view::format,
    },
};

/// Types the keys one after the other, each a character without modifiers
pub(super) fn typed(app: App, keys: &str) -> App {
    keys.chars().fold(app, |app, key| {
        app.typed(press(letter(&key.to_string())), false).0
    })
}

fn names(app: &App) -> Vec<&str> {
    app.workspace
        .outputs()
        .iter()
        .map(|output| output.name.as_str())
        .collect()
}

/// Returns the app with the outputs showing and the row counted from one
/// selected
pub(super) fn on(row: usize) -> App {
    after(outputs(), [Message::Row(row)])
}

fn locked(app: App) -> App {
    after(app, [Message::ShowMode(true)])
}

const ALL: [&str; 10] = [
    "House L",
    "House R",
    "Sub",
    "Monitor",
    "Front fill",
    "Balcony",
    "Lobby",
    "Stage",
    "FX",
    "Booth",
];

fn without(rows: &[usize]) -> Vec<&'static str> {
    ALL.iter()
        .enumerate()
        .filter(|(row, _)| !rows.contains(row))
        .map(|(_, name)| *name)
        .collect()
}

#[test]
fn o_and_capital_o_add_an_output_below_or_above_and_a_count_adds_that_many() {
    let below = typed(on(2), "o");
    assert_eq!(at(&below), 3);
    assert_eq!(
        names(&below)[..4],
        ["House L", "House R", "Output 1", "Sub"]
    );
    let above = after(on(2), [Message::Add(Place::Before, 1)]);
    assert_eq!(at(&above), 2);
    assert_eq!(names(&above)[..3], ["House L", "Output 1", "House R"]);
    // The last of those added is the one selected, as with cues
    let three = typed(on(2), "3o");
    assert_eq!(names(&three).len(), 13);
    assert_eq!(at(&three), 5);
    assert_eq!(names(&three)[2..5], ["Output 1", "Output 2", "Output 3"]);
    assert_eq!(three.workspace.cues().count(), 14, "no cue is added");
}

#[test]
fn a_count_of_added_outputs_stops_at_sixty_four() {
    let many = typed(on(1), "9999o");
    assert_eq!(many.workspace.outputs().len(), 10 + 64);
    // One change takes them all back
    let undone = typed(many, "u");
    assert_eq!(undone.workspace.outputs().len(), 10);
}

#[test]
fn the_keys_that_add_do_nothing_in_show_mode() {
    let kept = typed(locked(on(2)), "o");
    assert_eq!(names(&kept), ALL);
    assert_eq!(at(&kept), 2);
}

#[test]
fn dd_x_and_delete_cut_the_output_and_select_the_one_that_takes_its_place() {
    let cut = typed(on(3), "dd");
    assert_eq!(names(&cut), without(&[2]));
    assert_eq!(at(&cut), 3);
    assert_eq!(
        cut.clipped
            .as_ref()
            .map(|clipped| (clipped.told.as_str(), clipped.cut)),
        Some(("Cut the output Sub", true))
    );
    assert_eq!(names(&typed(on(3), "x")), without(&[2]));
    // The last output leaves the one before it selected
    assert_eq!(at(&typed(on(10), "dd")), 9);
    // Delete is the cut key of the vi keymap, and Control+x of cua
    let delete = chord(
        Key::Named(iced::keyboard::key::Named::Delete),
        Modifiers::empty(),
    );
    assert_eq!(names(&on(3).typed(delete, false).0), without(&[2]));
}

#[test]
fn counts_and_motions_cut_the_outputs_that_they_reach() {
    assert_eq!(names(&typed(on(2), "3dd")), without(&[1, 2, 3]));
    assert_eq!(names(&typed(on(2), "dj")), without(&[1, 2]));
    assert_eq!(names(&typed(on(4), "dk")), without(&[2, 3]));
    assert_eq!(names(&typed(on(3), "2dj")), without(&[2, 3, 4]));
    let cut = after(on(3), [Message::Across(Action::Cut, Action::Last, None)]);
    assert_eq!(names(&cut), ["House L", "House R"]);
    let first = after(
        on(3),
        [Message::Across(Action::Delete, Action::First, None)],
    );
    assert_eq!(names(&first), ALL[3..]);
    // One change takes the outputs back
    assert_eq!(names(&typed(typed(on(2), "3dd"), "u")), ALL);
}

#[test]
fn a_cut_that_would_take_every_output_does_nothing_and_says_so() {
    let refused = typed(on(1), "9999dd");
    assert_eq!(names(&refused), ALL);
    assert_eq!(refused.remark, Some("A show keeps at least one output"));
    assert!(refused.clipped.is_none());
    // The last output of a show that has one
    let one = after(on(1), [Message::Across(Action::Delete, Action::Last, None)]);
    assert_eq!(names(&one), ALL, "the cut reaches all of them");
    let single = typed(on(2), "dG");
    assert_eq!(names(&single), ["House L"]);
    let still = typed(single, "dd");
    assert_eq!(names(&still), ["House L"]);
    assert_eq!(still.remark, Some("A show keeps at least one output"));
}

#[test]
fn show_mode_locks_the_cut_and_delete_keys_and_fills_no_register() {
    for keys in ["dd", "x", "dj", "3dd"] {
        let kept = typed(locked(on(3)), keys);
        assert_eq!(names(&kept), ALL, "{keys}");
        assert!(kept.clipped.is_none(), "{keys}");
    }
    let deleted = after(locked(on(3)), [Message::Delete(1)]);
    assert_eq!(names(&deleted), ALL);
}

/// Returns the app with the cue 3 sending to the output Sub alone
fn sending_to_sub() -> App {
    let sends: Sends = [(OutputId(3), -4.0)].into_iter().collect();
    after(on(3), [Message::Edit(CueId(3), Edit::Sends(Some(sends)))])
}

#[test]
fn a_cut_output_pasted_back_takes_back_its_identity_as_dd_p_returns_a_line() {
    let cut = typed(sending_to_sub(), "dd");
    let cue_sends = |app: &App| {
        let cue = app.workspace.cue(CueId(3)).expect("the cue is there");
        app.workspace.sends(cue)
    };
    assert!(cue_sends(&cut).is_empty());
    // Pasting below the output that took its place puts it after that one
    let back = typed(typed(cut, "k"), "p");
    assert_eq!(names(&back), ALL);
    assert_eq!(at(&back), 3);
    assert_eq!(
        cue_sends(&back).into_iter().collect::<Vec<_>>(),
        [(OutputId(3), -4.0)]
    );
    // Pasting above puts it before the one that took its place
    let above = after(
        typed(sending_to_sub(), "dd"),
        [Message::Paste(Place::Before, 1)],
    );
    assert_eq!(names(&above), ALL);
}

#[test]
fn a_copy_pasted_is_an_output_of_its_own_with_a_free_name_and_no_sends() {
    let copied = typed(sending_to_sub(), "yy");
    assert_eq!(
        copied
            .clipped
            .as_ref()
            .map(|clipped| (clipped.told.as_str(), clipped.cut)),
        Some(("Copied the output Sub", false))
    );
    assert_eq!(names(&copied), ALL, "copying changes nothing");
    let pasted = typed(copied, "p");
    assert_eq!(names(&pasted)[2..5], ["Sub", "Sub 2", "Monitor"]);
    assert_eq!(at(&pasted), 4);
    let copy = pasted.workspace.outputs()[3].clone();
    assert_eq!(copy.new_cues, None);
    assert_ne!(copy.id, OutputId(3));
    let cue = pasted.workspace.cue(CueId(3)).expect("the cue is there");
    let sends: Vec<_> = pasted.workspace.sends(cue).into_iter().collect();
    assert_eq!(sends, [(OutputId(3), -4.0)], "no cue sends to the copy");
    // A count pastes that many, each under a name of its own
    let twice = typed(typed(on(3), "yy"), "2p");
    assert_eq!(names(&twice)[2..6], ["Sub", "Sub 2", "Sub 3", "Monitor"]);
    // Above, with capital P
    let above = after(typed(on(3), "yy"), [Message::Paste(Place::Before, 1)]);
    assert_eq!(names(&above)[2..4], ["Sub 2", "Sub"]);
}

#[test]
fn a_copy_with_the_motions_and_capital_y_takes_the_outputs_that_they_reach() {
    let two = typed(on(2), "yj");
    assert_eq!(
        two.clipped.as_ref().map(|clipped| clipped.told.as_str()),
        Some("Copied 2 outputs")
    );
    let pasted = typed(two, "p");
    assert_eq!(
        names(&pasted)[1..5],
        ["House R", "House R 2", "Sub 2", "Sub"]
    );
    let line = after(on(2), [Message::Copy(1)]);
    assert_eq!(
        line.clipped.as_ref().map(|clipped| clipped.told.as_str()),
        Some("Copied the output House R")
    );
}

#[test]
fn copying_acts_in_show_mode_and_pasting_does_not() {
    let copied = typed(locked(on(3)), "yy");
    assert!(copied.clipped.is_some());
    assert_eq!(names(&typed(copied, "p")), ALL);
}

#[test]
fn outputs_have_a_register_apart_from_cues() {
    // A cue copied in the cue list survives a copy of an output
    let cues = after(
        show(cue(2)),
        [Message::Copy(1), Message::ShowOutputs, Message::Copy(1)],
    );
    let back = after(
        cues,
        [Message::StepList(-1), Message::Paste(Place::After, 1)],
    );
    assert_eq!(back.workspace.cues().count(), 15, "the cue is pasted");
    // Nothing in the register of outputs pastes nothing, and tells so
    let empty = after(
        show(cue(2)),
        [
            Message::Copy(1),
            Message::ShowOutputs,
            Message::Paste(Place::After, 1),
        ],
    );
    assert_eq!(empty.workspace.outputs().len(), 1);
    assert_eq!(empty.remark, Some("The clipboard is empty"));
}

#[test]
fn a_cut_output_pasted_in_another_show_is_a_copy_there() {
    let cut = typed(sending_to_sub(), "dd");
    let register = Register {
        show: cut.show.id().wrapping_add(1),
        cut: true,
        outputs: cut.clipboard.outputs.outputs.clone(),
    };
    let elsewhere = App {
        clipboard: cut.clipboard.with_outputs(register),
        ..cut
    };
    let pasted = typed(typed(elsewhere, "k"), "p");
    let sub = &pasted.workspace.outputs()[2];
    assert_eq!(sub.name, "Sub");
    // The identity that the other show gave it means nothing here
    assert_ne!(sub.id, OutputId(3));
    let cue = pasted.workspace.cue(CueId(3)).expect("the cue is there");
    assert!(pasted.workspace.sends(cue).is_empty());
}

#[test]
fn a_cut_of_the_first_output_tells_where_cues_with_no_sends_play_now() {
    let cut = typed(on(1), "dd");
    let told = cut.clipped.as_ref().map(|clipped| clipped.told.clone());
    assert_eq!(
        told.as_deref(),
        Some("Cues without sends play on House R, cut House L")
    );
    // Not when the first output stays or no cue is left without sends
    let kept = typed(on(2), "dd");
    assert_eq!(
        kept.clipped.map(|clipped| clipped.told).as_deref(),
        Some("Cut the output House R")
    );
}

#[test]
fn a_change_of_the_first_output_tells_it_in_the_notice_of_the_change() {
    let notice = |app: &App| {
        let (_, change) = app.workspace.latest_change().expect("a change");
        let facts = format::consequences_of(&app.workspace, change);
        format::leading(facts, &format::change(&app.workspace, change))
    };
    let moved = after(on(1), [Message::Move(1)]);
    assert_eq!(
        notice(&moved),
        "Cues without sends play on House R, moved House L"
    );
    let added = after(on(1), [Message::Add(Place::Before, 1)]);
    assert_eq!(
        notice(&added),
        "Cues without sends play on Output 1, added Output 1"
    );
    let deleted = after(on(1), [Message::Delete(1)]);
    assert_eq!(
        notice(&deleted),
        "Cues without sends play on House R, deleted House L"
    );
    let calm = after(on(3), [Message::Move(1)]);
    assert_eq!(notice(&calm), "Moved the output Sub");
    let counted = after(on(3), [Message::Add(Place::After, 3)]);
    assert_eq!(notice(&counted), "Added 3 outputs");
}

#[test]
fn undo_and_redo_select_the_output_that_the_change_touched() {
    let cut = typed(on(3), "dd");
    let undone = typed(cut, "u");
    assert_eq!((names(&undone), at(&undone)), (ALL.to_vec(), 3));
    let redone = after(undone, [Message::Redo(1)]);
    assert_eq!((names(&redone), at(&redone)), (without(&[2]), 3));
    // A restored output is selected, wherever the keys left the selection
    let far = typed(typed(on(3), "dd"), "5j");
    assert_eq!(at(&typed(far, "u")), 3);
    // A move selects the output that moved, both ways
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
fn the_keys_that_enter_the_name_focus_the_field_of_the_output() {
    let started = |app: App, caret| app.handled(Message::Focus(Focus::Name(caret))).1.units();
    for caret in [Caret::End, Caret::Start, Caret::All] {
        assert!(started(on(2), caret) > 0, "{caret:?}");
        assert_eq!(started(locked(on(2)), caret), 0, "{caret:?}");
    }
    let hidden = App {
        output: None,
        ..on(2)
    };
    assert_eq!(started(hidden, Caret::End), 0);
}

#[test]
fn f2_of_the_cua_keymap_selects_the_name_of_the_output_and_tab_selects_the_next_field() {
    // The cua keymap binds F2 to this action, as its own tests show
    assert!(matches!(
        Action::ChangeName.message(None),
        Message::Focus(Focus::Name(Caret::All))
    ));
    // Moving by key starts the focus and then the selection of the text
    for focus in [Focus::Next, Focus::Previous] {
        assert!(on(2).handled(Message::Focus(focus)).1.units() > 1);
    }
}

#[test]
fn a_cut_by_keys_is_told_as_a_cut_by_the_change_and_a_delete_as_a_delete() {
    let told = |app: &App| {
        let (_, change) = app.workspace.latest_change().expect("a change");
        format::change(&app.workspace, change)
    };
    assert_eq!(told(&typed(on(3), "dd")), "cut the output Sub");
    assert_eq!(told(&typed(on(3), "2dj")), "cut 3 outputs");
    assert_eq!(
        told(&after(on(3), [Message::Delete(1)])),
        "deleted the output Sub"
    );
    // Leaving the outputs does not turn the cut into a delete
    let left = after(typed(on(3), "dd"), [Message::Back(1)]);
    assert_eq!(told(&left), "cut the output Sub");
}

#[test]
fn a_paste_by_keys_is_told_as_a_paste_and_an_add_as_an_add() {
    let told = |app: &App| {
        let (_, change) = app.workspace.latest_change().expect("a change");
        format::change(&app.workspace, change)
    };
    assert_eq!(told(&typed(on(3), "ddp")), "pasted the output Sub");
    assert_eq!(told(&typed(on(3), "yy2p")), "pasted 2 outputs");
    assert_eq!(
        told(&after(on(3), [Message::Add(Place::After, 1)])),
        "added the output Output 1"
    );
}

/// Types the keys as the window delivers them, one message each, which the
/// update cycle settles typed text for, unlike [`typed`]
fn keys(app: App, keys: &str) -> App {
    keys.chars().fold(app, |app, key| {
        let press = press(letter(&key.to_string()));
        app.next(Message::Typed {
            press,
            repeat: false,
        })
        .0
    })
}

#[test]
fn channels_typed_before_a_cut_apply_to_the_output_that_is_cut_and_pasted_back() {
    let sub = on(3).workspace.outputs()[2].id;
    let typing = after(
        on(3),
        [Message::SettingTyped(
            Setting::Channels(sub),
            "5-6".to_owned(),
        )],
    );
    assert_eq!(typing.workspace.outputs()[2].channels.to_string(), "1-2");
    // The key that cuts settles the text, and nothing of it is left to
    // apply to the output that has gone
    let cut = keys(typing, "dd");
    assert_eq!(cut.typed, None);
    assert_eq!(names(&cut), without(&[2]));
    let back = keys(cut, "P");
    let pasted = back
        .workspace
        .output(sub)
        .expect("the output took its identity back");
    assert_eq!(pasted.channels.to_string(), "5-6");
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
