// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the update cycle: the keys that act while the help page is
//! open, the help page that comes and goes without touching the main
//! window, the selection after deleting or undo takes the selected cue
//! away, the selection that follows standby, the sound file chosen for a
//! new cue, a level typed halfway, a link of the help page that fails to
//! open, the clipboard, and the keys that enter the name of a cue

mod adding;
mod cancel;
mod clipboard;
mod clipped;
mod clock;
mod counts;
mod help_page;
mod history;
mod loading;
mod matches;
mod naming;
mod operators;
mod operators_pasting;
mod paging;
mod scrolling;
mod search;
mod search_line;
mod search_places;
mod side_buttons;
mod viewing;

use std::ops::RangeInclusive;

use iced::{
    Task,
    keyboard::{Key, Modifiers, key::Named},
    window,
};

use crate::{
    back::{CueId, CueList, Place, Workspace, audio},
    front::{App, MAIN_SIZE, Message, Tab, keymap::Press},
};

/// Returns the state of the interface with `selected` selected in a show of
/// two lists of audio cues, numbered as they are identified: twelve in
/// Main, and two from twenty in Preshow
fn show(selected: Option<CueId>) -> App {
    let cues = |ids: RangeInclusive<u32>| ids.map(audio).collect();
    App {
        workspace: Workspace::new(vec![
            CueList::new("Main", cues(1..=12)),
            CueList::new("Preshow", cues(20..=21)),
        ]),
        selected,
        ..App::default()
    }
}

pub(super) fn chord(key: Key, modifiers: Modifiers) -> Press {
    Press::from_key(&key, modifiers).expect("keys in the tests are named")
}

pub(super) fn press(key: Key) -> Press {
    chord(key, Modifiers::empty())
}

/// Returns the keys that act in text fields, apart from those of GO and
/// Panic: those that open and save shows, go back and forward, and move
/// between fields
pub(super) fn field_keys() -> [Press; 7] {
    let tab = || Key::Named(Named::Tab);
    let control = |key: &str| chord(Key::Character(key.into()), Modifiers::CTRL);
    [
        control("l"),
        control("s"),
        control("S"),
        control("o"),
        control("i"),
        press(tab()),
        chord(tab(), Modifiers::SHIFT),
    ]
}

#[test]
fn deleting_selects_the_cue_after_or_else_the_one_before() {
    let deleting = |list: usize, selected: u32, count: usize| {
        let app = App {
            list,
            ..show(Some(CueId(selected)))
        };
        app.handled(Message::Delete(count)).0.selected
    };
    assert_eq!(deleting(0, 4, 2), Some(CueId(6)));
    assert_eq!(deleting(0, 11, 5), Some(CueId(10)));
    assert_eq!(deleting(1, 20, 2), None);
}

#[test]
fn undoing_away_the_selected_cue_selects_the_cue_in_its_place() {
    let (added, _) = show(Some(CueId(4))).handled(Message::Add(Place::Before, 1));
    let (undone, _) = added.handled(Message::Undo(1));
    assert_eq!(undone.selected, Some(CueId(4)));
}

#[test]
fn undoing_an_addition_or_a_paste_selects_the_cue_that_was_selected_when_it_was_made() {
    let selected_after = |messages: Vec<Message>| {
        let app = [Message::Select(CueId(4)), Message::Copy(1)]
            .into_iter()
            .chain(messages)
            .fold(show(Some(CueId(4))), |app, message| app.handled(message).0);
        app.selected
    };
    let undo = |message| selected_after(vec![message, Message::Undo(1)]);
    // Below and above, as `o u`, `O u`, `p u`, and `P u` of Vim end
    assert_eq!(undo(Message::Add(Place::After, 1)), Some(CueId(4)));
    assert_eq!(undo(Message::Add(Place::Before, 1)), Some(CueId(4)));
    assert_eq!(undo(Message::Paste(Place::After, 1)), Some(CueId(4)));
    assert_eq!(undo(Message::Paste(Place::Before, 1)), Some(CueId(4)));
    // Paste of several cues, and the selection moved before the undo
    let moved = selected_after(vec![
        Message::Paste(Place::After, 3),
        Message::Select(CueId(9)),
        Message::Undo(1),
    ]);
    assert_eq!(moved, Some(CueId(4)));
    // Redo selects what was added again
    let redone = selected_after(vec![
        Message::Add(Place::After, 1),
        Message::Undo(1),
        Message::Redo(1),
    ]);
    assert_eq!(redone, Some(CueId(22)));
}

#[test]
fn undoing_a_counted_addition_removes_all_its_cues_as_one_change_and_selects_the_anchor() {
    let counted = |place, count| {
        let (added, _) = show(Some(CueId(4))).handled(Message::Add(place, count));
        let cues: Vec<_> = added.workspace.lists()[0]
            .cues
            .iter()
            .map(|c| c.id.0)
            .collect();
        let (undone, _) = added.handled(Message::Undo(1));
        let after: Vec<_> = undone.workspace.lists()[0]
            .cues
            .iter()
            .map(|c| c.id.0)
            .collect();
        (cues, after, undone.selected)
    };
    let (cues, after, selected) = counted(Place::After, 3);
    assert_eq!(cues[3..8], [4, 22, 23, 24, 5]);
    assert_eq!(after, (1..=12).collect::<Vec<_>>());
    assert_eq!(selected, Some(CueId(4)));
    let (cues, after, selected) = counted(Place::Before, 3);
    assert_eq!(cues[2..7], [3, 22, 23, 24, 4]);
    assert_eq!(after, (1..=12).collect::<Vec<_>>());
    assert_eq!(selected, Some(CueId(4)));
    // One undo took all three, and redo puts them back with the first selected
    let (added, _) = show(Some(CueId(4))).handled(Message::Add(Place::After, 3));
    let (redone, _) = added.handled(Message::Undo(1)).0.handled(Message::Redo(1));
    assert_eq!(redone.workspace.lists()[0].cues.len(), 15);
    assert_eq!(redone.selected, Some(CueId(22)));
}

#[test]
fn sound_file_chosen_for_a_new_cue_names_it_by_its_path_from_the_show() {
    let app = App {
        workspace: Workspace::new(Vec::new()).directory("/show"),
        ..App::default()
    };
    let (added, _) = app.handled(Message::Add(Place::After, 1));
    let id = added.selected.expect("a new cue is selected");
    let chosen = Message::FileChosen(id, "/show/sounds/storm.ogg".into());
    let (chosen, _) = added.handled(chosen);
    let cue = chosen.workspace.cue(id).unwrap();
    assert_eq!(
        (cue.file.as_str(), cue.name.as_str()),
        ("sounds/storm.ogg", "storm")
    );
}

#[test]
fn standby_selected_without_a_selected_cue_leaves_standby_alone() {
    let app = show(None);
    let standby = app.workspace.lists()[0].standby;
    let (after, _) = app.handled(Message::StandbySelected);
    assert_eq!(after.workspace.lists()[0].standby, standby);
    assert!(after.workspace.undoable().is_none());
}

#[test]
fn selection_on_standby_moves_on_with_it_and_stays_elsewhere() {
    let going = |selected| show(Some(CueId(selected))).handled(Message::Go).0.selected;
    assert_eq!(going(1), Some(CueId(2)));
    assert_eq!(going(5), Some(CueId(5)));
}

#[test]
fn level_typed_so_far_shows_until_the_operator_does_anything_else() {
    let app = show(Some(CueId(1)));
    let typed = |text: &str| Message::LevelTyped(CueId(1), text.to_owned());
    let (half, _) = app.next(typed("-"));
    assert_eq!(half.typed_level, Some((CueId(1), "-".to_owned())));
    let (clicked, _) = half.next(Message::Clicked);
    assert!(clicked.typed_level.is_some());
    let (typed, _) = clicked.next(typed("-6"));
    assert_eq!(typed.workspace.cue(CueId(1)).unwrap().level, -6.0);
    let (resized, _) = typed.next(Message::Resized(window::Id::unique(), MAIN_SIZE));
    assert!(resized.typed_level.is_some());
    let (moved, _) = resized.next(Message::Step(1));
    assert_eq!(moved.typed_level, None);
}

#[test]
fn link_that_fails_to_open_shows_until_the_help_page_closes() {
    let failure = || Message::LinkOpened(Err("Cannot open it".to_owned()));
    let open = App {
        help: true,
        ..App::default()
    };
    let (failed, _) = open.next(failure());
    assert_eq!(failed.link_problem.as_deref(), Some("Cannot open it"));
    let (closed, _) = failed.next(Message::CloseHelp);
    assert_eq!(closed.link_problem, None);
    let (unseen, _) = closed.next(failure());
    assert_eq!(unseen.link_problem, None);
}
