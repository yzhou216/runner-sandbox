// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the update cycle, and the helpers that its test modules share

mod adding;
mod cancel;
mod clipboard;
mod clipped;
mod clock;
mod collapsed;
mod collapsed_expanding;
mod collapsed_search;
mod collapsed_standby;
mod counts;
mod field_runs;
mod grouping;
mod help_page;
mod hints;
mod history;
mod inputs;
mod loading;
mod matches;
mod naming;
mod new_kind;
mod operators;
mod output_editing;
mod output_moves;
mod output_places;
mod output_rows;
mod output_stranded;
mod outputs;
mod paging;
mod remote;
mod scrolling;
mod search;
mod search_columns;
mod search_line;
mod search_places;
mod selection;
mod show_mode;
mod side_buttons;
mod stopping;
mod typed_levels;
mod unapplied;
mod viewing;

use std::ops::RangeInclusive;

use iced::{
    Task,
    keyboard::{Key, Modifiers, key::Named},
    window,
};

use crate::{
    back::{Cue, CueId, CueList, Place, Workspace, audio, fresh, loaded, sound},
    front::{
        App, Message, Setting, Tab,
        app::Held,
        keymap::{Press, sequence},
        view::format,
    },
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

fn chord(key: Key, modifiers: Modifiers) -> Press {
    Press::from_key(&key, modifiers).expect("keys in the tests are named")
}

pub(super) fn press(key: Key) -> Press {
    chord(key, Modifiers::empty())
}

/// Returns how the latest change of the show reads in its notice
fn told(app: &App) -> String {
    let (_, change) = app.workspace.latest_change().expect("a change");
    format::change(&app.workspace, change)
}

/// Returns the notice of the latest change led by what it means for the cues
fn noticed(app: &App) -> String {
    let (_, change) = app.workspace.latest_change().expect("a change");
    let facts = format::consequences_of(&app.workspace, change);
    format::leading(facts, &format::change(&app.workspace, change))
}

/// Returns what the status bar tells of the last copy or cut, and whether it
/// was a cut
fn on_clipboard(app: &App) -> Option<(&str, bool)> {
    app.notices
        .clipped
        .as_ref()
        .map(|clipped| (clipped.told.as_str(), clipped.cut))
}

/// Returns the identities of the cues on the clipboard in order
fn held(app: &App) -> Vec<u32> {
    app.clipboard.cues.iter().map(|cue| cue.id.0).collect()
}

/// Returns the state with the help page open
fn helping(app: App) -> App {
    App { help: true, ..app }
}

/// Returns the state with a main window, and the window
fn windowed(app: App) -> (App, window::Id) {
    let main = window::Id::unique();
    (
        App {
            main: Some(main),
            ..app
        },
        main,
    )
}

/// Returns the state in show mode
fn locked(app: App) -> App {
    after(app, [Message::ShowMode(true)])
}

/// Returns the state of [`show`] with the Levels tab on screen
fn on_levels(selected: Option<CueId>) -> App {
    App {
        tab: Tab::Levels,
        ..show(selected)
    }
}

/// Returns the state of a show of one list of `cues` with `selected` selected
fn one_list(cues: Vec<Cue>, selected: Option<CueId>) -> App {
    App {
        workspace: fresh(cues),
        selected,
        ..App::default()
    }
}

/// Returns the state of a show of one list of `count` audio cues, the first
/// selected
fn crowded(count: u32) -> App {
    one_list((1..=count).map(audio).collect(), Some(CueId(1)))
}

/// Returns the state with the sound of cue 1 loaded, ten seconds long
fn with_sound(app: App) -> App {
    App {
        workspace: app.workspace.apply(loaded(1, "sound.ogg", sound(10.0))),
        ..app
    }
}

/// Returns the state of [`show`] with cue 1 selected and playing
fn playing() -> App {
    after(with_sound(show(Some(CueId(1)))), [Message::Go])
}

/// Returns the state after the messages, which a test sends one at a time
fn after(app: App, messages: impl IntoIterator<Item = Message>) -> App {
    messages
        .into_iter()
        .fold(app, |app, message| app.next(message).0)
}

/// Types the keys one after another, as the window delivers them, and returns
/// the state with the task that the last key started.  The keys are written as
/// the configuration file writes a sequence, apart by spaces, as in `3 o` or
/// `Control+r`.
fn typing_task(app: App, keys: &str) -> (App, Task<Message>) {
    sequence(keys)
        .expect("keys in the tests are valid")
        .into_iter()
        .fold((app, Task::none()), |(app, _), press| {
            app.typed(press, false)
        })
}

/// Returns the state after `text` goes into a field a key at a time, as the
/// widget reports it: each prefix of it as `message` makes it, with the
/// keyboard still in the field
fn typed_into(app: App, text: &str, message: impl Fn(String) -> Message) -> App {
    text.char_indices().fold(app, |app, (at, key)| {
        let prefix = text[..at + key.len_utf8()].to_owned();
        after(
            app,
            [
                message(prefix),
                Message::Held(Held {
                    any: true,
                    ..Held::default()
                }),
            ],
        )
    })
}

/// Returns the state after `text` is typed into the field of `setting`
fn typed_setting(app: App, setting: Setting, text: &str) -> App {
    after(app, [Message::SettingTyped(setting, text.to_owned())])
}

/// Returns the state after the keys of [`typing_task`]
pub(super) fn typing(app: App, keys: &str) -> App {
    typing_task(app, keys).0
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
