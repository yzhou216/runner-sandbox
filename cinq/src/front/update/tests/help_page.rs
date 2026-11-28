// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the help page: the keys that close it, scroll it, and act over
//! it, and how it comes and goes

use super::*;

fn stays_open(key: Key) -> bool {
    let open = helping(App::default());
    open.typed(press(key), false).0.help
}

#[test]
fn help_page_closes_on_its_keys_and_on_go_and_panic() {
    assert!(!stays_open(Key::Character("q".into())));
    assert!(!stays_open(Key::Named(Named::F1)));
    assert!(!stays_open(Key::Character("?".into())));
    assert!(!stays_open(Key::Named(Named::Space)));
    assert!(!stays_open(Key::Named(Named::Escape)));
    assert!(stays_open(Key::Character("j".into())));
}

#[test]
fn keys_that_move_the_selection_scroll_the_open_help_page() {
    let typed = |keys, help| {
        let app = App {
            help,
            ..show(Some(CueId(2)))
        };
        let (after, task) = typing_task(app, keys);
        (after.selected, task.units() > 0)
    };
    // Scrolling moves no selection, which the keys move with the page shut
    ["j", "k", "G"]
        .into_iter()
        .for_each(|key| assert_eq!(typed(key, true), (Some(CueId(2)), true), "{key}"));
    assert_eq!(typed("g g", true), (Some(CueId(2)), true));
    assert_eq!(typed("j", false).0, Some(CueId(3)));
    // Other keys that act on the cues do nothing over the page
    assert_eq!(typed("o", true), (Some(CueId(2)), false));
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
    field_keys()[5..].iter().for_each(|&key| {
        let (_, acting) = typed(false, key);
        assert!(acting.units() > 0, "{key:?}");
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
fn the_key_of_standby_opens_the_tutorial_once_from_the_help_page_and_leaves_it_open() {
    let enter = press(Key::Named(Named::Enter));
    let page = |help| App { help, ..show(None) };
    let (open, task) = page(true).typed(enter, false);
    assert_eq!((open.help, task.units() > 0), (true, true));
    // With the page shut the key moves standby and opens nothing
    let (_, task) = page(false).typed(enter, false);
    assert_eq!(task.units(), 0);
    // Held key opens it once
    let (_, repeat) = page(true).typed(enter, true);
    assert_eq!(repeat.units(), 0);
}

#[test]
fn holding_the_key_of_help_leaves_the_page_as_it_is() {
    [Key::Named(Named::F1), Key::Character("?".into())]
        .into_iter()
        .for_each(|key| {
            assert!(!App::default().typed(press(key.clone()), true).0.help);
            assert!(helping(App::default()).typed(press(key), true).0.help);
        });
}

#[test]
fn holding_the_key_of_open_or_save_opens_one_file_dialog() {
    ["Control+l", "Control+s", "Control+S"]
        .into_iter()
        .for_each(|keys| {
            let key = sequence(keys)
                .expect("keys in the tests are valid")
                .remove(0);
            assert!(App::default().typed(key, false).1.units() > 0, "{keys}");
            assert_eq!(App::default().typed(key, true).1.units(), 0, "{keys}");
        });
}
