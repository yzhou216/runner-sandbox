// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the outputs view: reaching it past the last cue list, the keys
//! of cues acting on outputs there, the messages for rows that act on nothing
//! there, and the settings typed into fields

use std::time::Instant;

use super::show;
use crate::{
    back::{CueId, OutputId, Place},
    front::{
        App, Focus, Message, Setting,
        keymap::Action,
        message::{Height, LineKey, Page},
        update::Spot,
    },
};

fn names(app: &App) -> Vec<&str> {
    app.workspace
        .outputs()
        .iter()
        .map(|output| output.name.as_str())
        .collect()
}

fn after(app: App, messages: impl IntoIterator<Item = Message>) -> App {
    messages
        .into_iter()
        .fold(app, |app, message| app.handled(message).0)
}

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
    format!("{lists:?} {} {:?} {:?}", app.list, app.selected, app.tab,)
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
        Message::SearchKey(LineKey::DeleteAll),
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

#[test]
fn going_back_from_the_outputs_returns_to_the_cue_list_that_showed_before() {
    let outputs = after(
        show(Some(CueId(3))),
        [Message::Select(CueId(21)), Message::ShowOutputs],
    );
    let back = after(outputs, [Message::Back(1)]);
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

fn typed(app: App, setting: Setting, text: &str) -> App {
    after(app, [Message::SettingTyped(setting, text.to_owned())])
}

#[test]
fn typed_settings_apply_once_they_read_and_show_as_typed_meanwhile() {
    let main = OutputId(1);
    let app = after(show(None), [Message::ShowOutputs]);
    let app = typed(app, Setting::NewCues(main), "-");
    assert_eq!(app.workspace.outputs()[0].new_cues, Some(0.0));
    let app = typed(app, Setting::NewCues(main), "-6");
    assert_eq!(app.workspace.outputs()[0].new_cues, Some(-6.0));
    let app = typed(app, Setting::NewCues(main), "");
    assert_eq!(app.workspace.outputs()[0].new_cues, None);
}

fn channels(app: &App) -> String {
    app.workspace.outputs()[0].channels.to_string()
}

#[test]
fn channels_apply_at_enter_not_at_each_prefix_that_reads() {
    let main = OutputId(1);
    let app = after(show(None), [Message::ShowOutputs]);
    // Each of 1, 12, and 12-13 reads, and 12 would ask a device for 12
    // channels on the way to the pair
    let app = ["1", "12", "12-", "12-13"]
        .into_iter()
        .fold(app, |app, text| typed(app, Setting::Channels(main), text));
    assert_eq!(channels(&app), "1-2");
    assert_eq!(
        app.typed,
        Some((Setting::Channels(main), "12-13".to_owned()))
    );
    // Enter in the field gives the keyboard to the cue list, as it does in
    // every other text field, and that applies the channels
    let app = app.next(Message::Focus(Focus::Cues)).0;
    assert_eq!(channels(&app), "12-13");
    assert_eq!(app.typed, None);
}

#[test]
fn channels_apply_when_the_operator_goes_on_to_anything_else() {
    let main = OutputId(1);
    let typing = |text: &str| {
        let app = after(show(None), [Message::ShowOutputs]);
        typed(app, Setting::Channels(main), text)
    };
    let next = |app: App, message| app.next(message).0;
    // Another field, a key outside any field, a click, or any other message
    let field = typed(typing("3-4"), Setting::OutputName(main), "House");
    assert_eq!(channels(&field), "3-4");
    let elsewhere = next(typing("3-4"), Message::Step(1));
    assert_eq!(channels(&elsewhere), "3-4");
    assert_eq!(elsewhere.typed, None);
    let clicked = next(typing("3-4"), Message::Clicked);
    assert_eq!(channels(&clicked), "3-4", "a click anywhere settles them");
    assert_eq!(clicked.typed, None, "so that no key after it applies them");
    let tick = next(typing("3-4"), Message::Tick(Instant::now()));
    assert_eq!(channels(&tick), "1-2", "a tick is not the operator");
    assert_eq!(channels(&next(typing("3-"), Message::Step(1))), "1-2");
}

#[test]
fn typed_sends_apply_and_an_empty_field_takes_the_send_away() {
    let main = OutputId(1);
    let cue = CueId(1);
    let sends = |app: &App| {
        let cue = app.workspace.cue(cue).expect("the cue is there");
        app.workspace.sends(cue).get(&main).copied()
    };
    let app = typed(show(None), Setting::Send(cue, main), "-6");
    assert_eq!(sends(&app), Some(-6.0));
    let app = typed(app, Setting::Send(cue, main), "");
    assert_eq!(sends(&app), None);
}

#[test]
fn editing_a_send_keeps_the_sends_to_outputs_that_have_gone() {
    let (main, sub, cue) = (OutputId(1), OutputId(2), CueId(1));
    let app = after(
        show(None),
        [Message::ShowOutputs, Message::Add(Place::After, 1)],
    );
    let app = typed(app, Setting::Send(cue, sub), "-3");
    let app = after(app, [Message::Delete(1)]);
    assert_eq!(names(&app), ["Main"]);
    let app = typed(app, Setting::Send(cue, main), "-6");
    let kept = app.workspace.cue(cue).and_then(|cue| cue.sends.clone());
    assert_eq!(kept.and_then(|sends| sends.get(&sub).copied()), Some(-3.0));
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

fn outputs_after_list_two() -> App {
    after(
        show(Some(CueId(3))),
        [Message::Select(CueId(21)), Message::ShowOutputs],
    )
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
fn the_sidebar_lights_only_while_a_menu_is_open_with_its_highlight_on_the_default() {
    let lit = |app: &App| app.menu && app.default_highlighted;
    let open = after(
        show(None),
        [Message::MenuOpen(true), Message::DefaultHighlighted(true)],
    );
    assert!(lit(&open));
    let moved = after(open, [Message::DefaultHighlighted(false)]);
    assert!(!lit(&moved));
    let closed = after(
        after(
            show(None),
            [Message::MenuOpen(true), Message::DefaultHighlighted(true)],
        ),
        [Message::MenuOpen(false)],
    );
    assert!(!lit(&closed));
    assert!(
        !closed.default_highlighted,
        "the app never thinks that the highlight rests there once the menu closes"
    );
}

#[test]
fn any_menu_opening_or_closing_clears_the_highlight_that_the_sidebar_lights_for() {
    let open = after(
        show(None),
        [Message::MenuOpen(true), Message::DefaultHighlighted(true)],
    );
    assert!(
        !after(open, [Message::MenuOpen(true)]).default_highlighted,
        "the menu of another list opens while the flag is still set"
    );
    let stale = App {
        default_highlighted: true,
        ..show(None)
    };
    assert!(!after(stale, [Message::MenuOpen(false)]).default_highlighted);
}

#[test]
fn enter_in_a_field_that_reads_no_value_leaves_it_and_tells_why() {
    let main = OutputId(1);
    let app = after(show(None), [Message::ShowOutputs]);
    let app = typed(app, Setting::Channels(main), "x");
    let app = app.next(Message::Focus(Focus::Cues)).0;
    assert_eq!(channels(&app), "1-2");
    assert_eq!(app.typed, None);
    assert!(app.unapplied.is_some());
}
