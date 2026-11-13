// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the update cycle: the keys that act while the help page is
//! open, the help page that comes and goes without touching the main
//! window, the selection after deleting or undo takes the selected cue
//! away, the selection that follows standby, the sound file chosen for a
//! new cue, a level typed halfway, a link of the help page that fails to
//! open, loading the files of the show, and the window drawn before a show
//! has opened

mod clock;

use std::ops::RangeInclusive;

use iced::{
    Task,
    keyboard::{Key, Modifiers, key::Named},
    window,
};

use crate::{
    back::{CueId, CueList, Load, Place, Workspace, audio, load_files},
    front::{App, Entry, MAIN_SIZE, Message, Tab, keymap::Press},
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
/// Panic: those that move between fields, and open and save shows
pub(super) fn field_keys() -> [Press; 5] {
    let tab = || Key::Named(Named::Tab);
    let control = |key: &str| chord(Key::Character(key.into()), Modifiers::CTRL);
    [
        control("o"),
        control("s"),
        control("S"),
        press(tab()),
        chord(tab(), Modifiers::SHIFT),
    ]
}

fn stays_open(key: Key) -> bool {
    let open = App {
        help: true,
        ..App::default()
    };
    open.typed(press(key), false).0.help
}

#[test]
fn help_page_closes_on_its_keys_and_on_go_and_panic() {
    assert!(!stays_open(Key::Character("q".into())));
    assert!(!stays_open(Key::Character("?".into())));
    assert!(!stays_open(Key::Named(Named::Space)));
    assert!(!stays_open(Key::Named(Named::Escape)));
    assert!(stays_open(Key::Character("j".into())));
}

#[test]
fn keys_that_move_the_selection_scroll_the_open_help_page() {
    let typed = |keys: &[&str], help: bool| {
        let app = App {
            help,
            ..show(Some(CueId(2)))
        };
        let (after, task) = keys.iter().fold((app, Task::none()), |(app, _), &key| {
            app.typed(press(Key::Character(key.into())), false)
        });
        (after.selected, task.units() > 0)
    };
    // Scrolling moves no selection, which the keys move with the page shut
    ["j", "k", "G"]
        .into_iter()
        .for_each(|key| assert_eq!(typed(&[key], true), (Some(CueId(2)), true), "{key}"));
    assert_eq!(typed(&["g", "g"], true), (Some(CueId(2)), true));
    assert_eq!(typed(&["j"], false).0, Some(CueId(3)));
    // Other keys that act on the cues do nothing over the page
    assert_eq!(typed(&["o"], true), (Some(CueId(2)), false));
}

#[test]
fn keys_that_act_in_fields_do_nothing_over_the_help_page_but_go_still_fires() {
    let typed = |help: bool, press| App { help, ..show(None) }.typed(press, false);
    field_keys().into_iter().for_each(|key| {
        let (over, ignored) = typed(true, key);
        assert_eq!((over.help, ignored.units()), (true, 0), "{key:?}");
    });
    // Keys of opening and saving would build a file chooser, which rfd
    // builds only on the main thread on Darwin, so only the moves between
    // fields are fired with the page shut
    field_keys()[3..].iter().for_each(|&key| {
        let (_, acting) = typed(false, key);
        assert_eq!(acting.units(), 1, "{key:?}");
    });
    let standby = |app: &App| app.workspace.lists()[0].standby;
    let (open, _) = typed(true, press(Key::Named(Named::Space)));
    assert_eq!(standby(&show(None)), Some(CueId(1)));
    assert_eq!((open.help, standby(&open)), (false, Some(CueId(2))));
}

#[test]
fn help_page_comes_and_goes_without_touching_the_main_window() {
    // What iced keeps in widgets, the field with the keyboard and the
    // scroll of the cue list, stays in the widgets that the view hides
    // under the page, so that no task has anything to save or to restore
    let app = App {
        list: 1,
        tab: Tab::Levels,
        ..show(Some(CueId(20)))
    };
    let (open, opening) = app.next(Message::ToggleHelp);
    assert_eq!((open.help, opening.units()), (true, 0));
    let (closed, closing) = open.next(Message::CloseHelp);
    assert_eq!((closed.help, closing.units()), (false, 0));
    assert_eq!(
        (closed.list, closed.tab, closed.selected),
        (1, Tab::Levels, Some(CueId(20)))
    );
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
    let (added, _) = show(Some(CueId(4))).handled(Message::Add(Place::After));
    let (undone, _) = added.handled(Message::Undo);
    assert_eq!(undone.selected, Some(CueId(5)));
}

#[test]
fn sound_file_chosen_for_a_new_cue_names_it_by_its_path_from_the_show() {
    let app = App {
        workspace: Workspace::new(Vec::new()).directory("/show"),
        ..App::default()
    };
    let (added, _) = app.handled(Message::Add(Place::After));
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
    let level = Entry::Level(CueId(1));
    let typed = |text: &str| Message::Entered(level, text.to_owned());
    let (half, _) = app.next(typed("-"));
    assert_eq!(half.entered, Some((level, "-".to_owned())));
    let (clicked, _) = half.next(Message::Clicked);
    assert!(clicked.entered.is_some());
    let (typed, _) = clicked.next(typed("-6"));
    assert_eq!(typed.workspace.cue(CueId(1)).unwrap().level, -6.0);
    let (resized, _) = typed.next(Message::Resized(window::Id::unique(), MAIN_SIZE));
    assert!(resized.entered.is_some());
    let (moved, _) = resized.next(Message::Step(1));
    assert_eq!(moved.entered, None);
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

#[test]
fn one_load_runs_at_a_time_until_what_it_found_comes_back() {
    let app = show(None);
    let found = load_files(Load::all(&app.workspace, app.workspace.unloaded()));
    let (loading, first) = app.load();
    let (still, again) = loading.load();
    assert_eq!(first.units(), 1);
    assert_eq!(again.units(), 0);
    assert!(still.loading);
    let show = still.show.id();
    let done = still.loaded(show, found);
    assert!(!done.loading);
    assert_eq!(done.workspace.unloaded().count(), 0);
}

#[test]
fn window_draws_before_a_show_has_opened() {
    let _ = crate::front::view::view(&App::default());
}
