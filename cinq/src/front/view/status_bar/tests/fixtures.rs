// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Apps, shows, and hints that the tests of the status bar share

use unicode_width::UnicodeWidthStr;

use super::super::{fit::fitted, hints::hints as hinted, order::yielding, status};
use crate::{
    back::{Command, CueId, Kind, Place, Workspace, fresh, memo},
    front::{
        App,
        app::{Held, Keyboard, Notices},
        config::Config,
        keymap::{Action, Keymap},
        update::Clipped,
    },
};

pub(super) fn texts(app: &App) -> Vec<String> {
    hinted(app).into_iter().map(|(_, hint)| hint).collect()
}

pub(super) fn taken(app: &App) -> usize {
    status(app, app.show.name().into_owned())
        .iter()
        .map(|part| part.text.width())
        .sum()
}

pub(super) fn add() -> Command {
    Command::Add {
        list: 0,
        kind: Kind::Audio,
        at: Place::End,
        count: 1,
    }
}

pub(super) fn added() -> Workspace {
    Workspace::default().apply(add())
}

pub(super) fn deleted() -> Workspace {
    fresh(vec![memo(1)]).apply(Command::Delete(vec![CueId(1)]))
}

pub(super) fn owned<const N: usize>(
    hints: [(Option<Action>, &str); N],
) -> Vec<(Option<Action>, String)> {
    hints
        .into_iter()
        .map(|(action, hint)| (action, hint.to_owned()))
        .collect()
}

/// Returns the hints after a change, with unsaved changes
pub(super) fn hints() -> Vec<(Option<Action>, String)> {
    owned([
        (None, "Added cue 1 | u: undo"),
        (Some(Action::Save), "Control+s: save"),
        (Some(Action::Go), "Space: GO"),
        (Some(Action::Panic), "Escape: Panic"),
        (Some(Action::Help), "?: help"),
    ])
}

pub(super) fn opened(menu: bool, help: bool) -> App {
    opened_with(Keymap::default(), menu, help)
}

/// Returns a keyboard that a text field or a drop-down list has when `any`
pub(super) fn holding(any: bool) -> Keyboard {
    Keyboard::default().with_held(Held {
        any,
        ..Held::default()
    })
}

pub(super) fn keyed(app: App, change: impl FnOnce(Keyboard) -> Keyboard) -> App {
    App {
        keyboard: change(app.keyboard),
        ..app
    }
}

pub(super) fn opened_with(keymap: Keymap, menu: bool, help: bool) -> App {
    let mut config = Config::default();
    config.keymap = keymap;
    App {
        config,
        keyboard: Keyboard {
            menu,
            ..Keyboard::default()
        },
        help,
        ..App::default()
    }
}

pub(super) fn has_hint(app: &App, hint: &str) -> bool {
    hinted(app).iter().any(|(_, told)| told == hint)
}

pub(super) fn hints_fitted(app: &App, room: usize) -> String {
    fitted(&hinted(app), room, yielding(app))
}

/// Returns four apps made by `make`, in which a field, the help page, a menu,
/// or the outputs view has the keys
pub(super) fn covered(make: impl Fn() -> App) -> [App; 4] {
    [
        App {
            keyboard: holding(true),
            ..make()
        },
        App {
            help: true,
            ..make()
        },
        keyed(make(), |keyboard| keyboard.with_menu(true)),
        App {
            outputs_shown: true,
            ..make()
        },
    ]
}

/// Hint of the key of GO, for the lists that `before` makes
pub(super) const GO: (Action, &str) = (Action::Go, "Space: GO");

/// Returns a function that lists the notice that it takes, if any, before the
/// `hints` of `keys`
pub(super) fn before<const N: usize>(
    keys: [(Action, &'static str); N],
) -> impl Fn(Option<String>) -> Vec<(Option<Action>, String)> {
    move |told| {
        told.into_iter()
            .map(|told| (None, told))
            .chain(
                keys.iter()
                    .map(|&(action, hint)| (Some(action), hint.to_owned())),
            )
            .collect()
    }
}

/// Returns the app of a cut or a copy that told `told`, in `workspace`
pub(super) fn clipping(told: &str, cut: bool, workspace: Workspace) -> App {
    App {
        notices: Notices {
            clipped: Some(Clipped {
                told: told.to_owned(),
                cut,
            }),
            ..Notices::default()
        },
        workspace,
        ..App::default()
    }
}

/// Returns an app under `keymap` in which a list has the keyboard, or a text
/// field does
pub(super) fn held(keymap: Keymap, text: bool) -> App {
    App {
        keyboard: Keyboard::default().with_held(Held {
            any: true,
            text,
            ..Held::default()
        }),
        ..opened_with(keymap, false, false)
    }
}
