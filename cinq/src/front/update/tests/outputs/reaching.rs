// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of reaching and leaving the outputs and of the keys and messages there

use super::super::{after, output_editing::names, show};
use crate::{
    back::{CueId, OutputId, Place},
    front::{
        App, Message,
        keymap::Action,
        message::{Height, LineKey, Page, Spot},
    },
};

#[test]
fn outputs_lie_past_the_last_cue_list_and_back() {
    let shown = after(
        show(Some(CueId(1))),
        [Message::StepList(1), Message::StepList(1)],
    );
    assert!(shown.outputs_shown);
    assert_eq!(shown.output, Some(OutputId(1)));
    let past = after(shown, [Message::StepList(1)]);
    assert!(past.outputs_shown, "nothing lies past the outputs");
    let back = after(past, [Message::StepList(-1)]);
    assert!(!back.outputs_shown);
    assert_eq!(back.list, 1);
}

#[test]
fn keys_of_cues_add_move_and_delete_outputs_while_they_show() {
    let added = after(
        show(None),
        [
            Message::ShowOutputs,
            Message::Add(Place::After, 1),
            Message::Add(Place::Before, 1),
        ],
    );
    assert_eq!(names(&added), ["Main", "Output 2", "Output 1"]);
    assert_eq!(added.output, Some(OutputId(3)), "the new output shows");
    assert_eq!(added.workspace.cues().count(), 14, "no cue is added");
    let moved = after(added, [Message::Move(-1)]);
    assert_eq!(names(&moved), ["Output 2", "Main", "Output 1"]);
    let deleted = after(moved, [Message::Delete(1)]);
    assert_eq!(names(&deleted), ["Main", "Output 1"]);
    assert_eq!(deleted.output, Some(OutputId(1)), "the next output shows");
}

/// Returns what a message for the rows could change in a cue list that is out
/// of sight
fn cue_list(app: &App) -> String {
    let lists: Vec<_> = app
        .workspace
        .lists()
        .iter()
        .map(|list| {
            (
                list.standby,
                list.cues.iter().map(|cue| cue.id).collect::<Vec<_>>(),
            )
        })
        .collect();
    format!("{lists:?} {} {:?} {:?}", app.list, app.selected, app.tab)
}

#[test]
fn no_message_for_rows_reaches_the_cue_list_while_the_outputs_show() {
    let rows = [
        Message::Step(1),
        Message::Row(2),
        Message::Page(Page::HalfDown, None),
        Message::Landed {
            from: Spot::Cue(CueId(3)),
            row: 5,
        },
        Message::Scroll(2),
        Message::InView(Height::Top, None),
        Message::Reached {
            from: Spot::Cue(CueId(3)),
            row: 5,
        },
        Message::Place(Height::Middle, None),
        Message::StandbySelected,
        Message::StepTab(1),
        Message::ShowTab(2),
        Message::Add(Place::After, 2),
        Message::Move(1),
        Message::Copy(2),
        Message::Cut(2),
        Message::Paste(Place::Before, 2),
        Message::Delete(2),
        Message::Across(Action::Delete, Action::Next, None),
        Message::Search { backward: false },
        Message::Searching("a".to_owned()),
        Message::SearchKey(LineKey::Older),
        Message::SearchAccepted,
        Message::SearchStep(1),
    ];
    let shown = || after(show(Some(CueId(3))), [Message::ShowOutputs]);
    let before = cue_list(&shown());
    rows.into_iter().for_each(|message| {
        assert!(message.acts_on_rows(), "{message:?}");
        let app = shown().handled(message.clone()).0;
        assert!(app.outputs_shown, "{message:?}");
        assert_eq!(cue_list(&app), before, "{message:?}");
    });
}

/// Returns the interface with the outputs shown after list 2, cue 21 selected
fn outputs_after_list_two() -> App {
    after(
        show(Some(CueId(3))),
        [Message::Select(CueId(21)), Message::ShowOutputs],
    )
}

#[test]
fn going_back_from_the_outputs_returns_to_the_cue_list_that_showed_before() {
    let back = after(outputs_after_list_two(), [Message::Back(1)]);
    assert!(!back.outputs_shown);
    assert_eq!((back.list, back.selected), (1, Some(CueId(21))));
    let back = after(back, [Message::Back(1)]);
    assert_eq!((back.list, back.selected), (0, Some(CueId(3))));
    let switched = after(
        show(Some(CueId(3))),
        [Message::ShowOutputs, Message::SwitchList(1)],
    );
    assert!(!switched.outputs_shown);
    let back = after(switched, [Message::Back(1)]);
    assert!(back.outputs_shown);
    let back = after(back, [Message::Back(1)]);
    assert_eq!((back.list, back.selected), (0, Some(CueId(3))));
}

#[test]
fn warning_about_an_output_shows_the_outputs_with_it_selected() {
    let added = after(
        show(None),
        [Message::ShowOutputs, Message::Add(Place::After, 1)],
    );
    let back = after(added, [Message::StepList(-1)]);
    assert!(!back.outputs_shown);
    let inspected = after(back, [Message::InspectOutput(OutputId(2))]);
    assert!(inspected.outputs_shown);
    assert_eq!(inspected.output, Some(OutputId(2)));
}

#[test]
fn leaving_the_outputs_for_the_list_that_showed_keeps_its_selected_cue() {
    let previous = after(outputs_after_list_two(), [Message::StepList(-1)]);
    assert!(!previous.outputs_shown);
    assert_eq!((previous.list, previous.selected), (1, Some(CueId(21))));
    let sidebar = after(outputs_after_list_two(), [Message::SwitchList(1)]);
    assert_eq!((sidebar.list, sidebar.selected), (1, Some(CueId(21))));
}

#[test]
fn leaving_the_outputs_for_another_list_selects_its_standby_cue() {
    let other = after(outputs_after_list_two(), [Message::SwitchList(0)]);
    assert_eq!(other.selected, other.workspace.lists()[0].standby);
    let standing = after(show(Some(CueId(3))), [Message::SwitchList(0)]);
    assert_eq!(
        standing.selected,
        standing.workspace.lists()[0].standby,
        "switching to the list that shows still selects standby"
    );
}

#[test]
fn a_list_past_the_last_is_left_alone() {
    let app = after(show(Some(CueId(3))), [Message::SwitchList(2)]);
    assert_eq!((app.list, app.selected), (0, Some(CueId(3))));
}
