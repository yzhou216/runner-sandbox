// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Status of the show, most important first, then at the right the keys of
//! a command typed so far, what the latest opening or saving told or else
//! the latest change to the show with the key that takes it back, the keys
//! that go back and forward between the places jumped to while there is a
//! place to go to, the key that saves unsaved changes, and the keys of GO,
//! Panic, and the help page
//!
//! The bar keeps to one line.  When it is too narrow for all of it, the
//! keys of going forward and back, of GO, of Panic, of saving, and of the
//! help page give way in turn, each one whole, and then the status gives way
//! from its end, so that the keys typed so far and what the latest change
//! told always show in full.
//! An open search line takes the place of the status, and the hints give way
//! to its text as well.

#[cfg(test)]
mod tests;

use std::iter;

use iced::{
    Center, Element, Fill, Padding,
    widget::{container, responsive, rich_text, row, span, text, text::Wrapping, text_input},
};
use itertools::Itertools;
use unicode_width::UnicodeWidthStr;

use super::{
    LINE, SEARCH, SPACING, answered, char_width, format, giving_way, keep_keys, line_height, theme,
    warnings,
};
use crate::{
    back::Turn,
    front::{
        App, Message, Search,
        keymap::{Action, written},
    },
};

const YIELDING: [Action; 8] = [
    Action::Forward,
    Action::Back,
    Action::Go,
    Action::Panic,
    Action::Save,
    Action::Help,
    Action::CloseMenu,
    Action::Pick,
];

/// Mark that opens the search line, or the one that opens it for a search that
/// goes up the list, as in Vim
fn mark(backward: bool) -> &'static str {
    if backward { "?" } else { "/" }
}

/// What the search line tells when no cue matches the text
const NO_MATCH: &str = "No cue matches";

/// Fewest columns that the search line keeps for itself, which the hints
/// give way to
const SEARCH_ROOM: usize = 24;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    // Bar is one line of text high with the gap around it, since a
    // responsive widget fills all the height it is given
    let height = line_height() + 2.0 * SPACING;
    container(responsive(move |size| bar(app, size.width)))
        .height(height)
        .into()
}

/// Hints grow to the left, so that the keys typed so far come and go
/// without moving the rest.  An open search line takes the place of the
/// status, as Vim's command line does, and the hints give way to its text.
fn bar(app: &App, width: f32) -> Element<'_, Message> {
    let room = ((width - 3.0 * SPACING) / char_width()).floor() as usize;
    let (left, taken) = match &app.search {
        Some(search) => (
            search_line(app, search),
            search_columns(&search.text, search.warned),
        ),
        None => {
            let status = status(app);
            let columns = status.iter().map(|part| part.text.width()).sum();
            let shown = container(rich_text(status).wrapping(Wrapping::None))
                .width(Fill)
                .clip(true);
            (shown.into(), columns)
        }
    };
    let shown = fitted(&hints(app), room.saturating_sub(taken));
    {
        thread_local!(static LAST: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) });
        LAST.with(|last| {
            if *last.borrow() != shown {
                eprintln!(
                    "SANDBOX {:.3} status hints={shown:?} room={room} menu={}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0.0, |time| time.as_secs_f64()),
                    app.menu
                );
                last.replace(shown.clone());
            }
        });
    }
    row![left, text(shown).wrapping(Wrapping::None)]
        .spacing(SPACING)
        .padding(SPACING)
        .align_y(Center)
        .into()
}

/// Columns that the search line keeps for its text, and for what it tells,
/// so that the hints never cover them
fn search_columns(text: &str, warned: bool) -> usize {
    let told = if warned { NO_MATCH.width() + 1 } else { 0 };
    (mark(false).width() + text.width() + told).max(SEARCH_ROOM)
}

/// Search line: its mark, the text typed, and a note that no cue matches
/// for a while after a search finds none.  It keeps the keys typed into
/// it, as the fields of the inspector do.
fn search_line<'a>(app: &'a App, search: &'a Search) -> Element<'a, Message> {
    let field = text_input("", &search.text)
        .id(SEARCH)
        .on_input(Message::Searching)
        .on_submit(Message::SearchAccepted)
        .padding(0)
        .style(theme::line);
    let line = row![text(mark(search.backward)), field]
        .push(search.warned.then(|| {
            container(
                text(NO_MATCH)
                    .color(theme::PALETTE.warning)
                    .wrapping(Wrapping::None),
            )
            .padding(Padding::ZERO.right(SPACING))
        }))
        .align_y(Center);
    keep_keys(line, &app.config.keymap)
}

/// Returns the hints joined, each one whole, without as many of those
/// that give way as it takes for the rest to fit in `room` columns
fn fitted(hints: &[(Option<Action>, String)], room: usize) -> String {
    giving_way(
        &YIELDING,
        |gone| {
            hints
                .iter()
                .filter(|(action, _)| !action.is_some_and(|action| gone.contains(&action)))
                .map(|(_, hint)| hint)
                .join(LINE)
        },
        |hints| hints.width() <= room,
    )
}

fn status(app: &App) -> Vec<text::Span<'_>> {
    let count = warnings(app);
    let mode = if app.editable() {
        "Edit mode"
    } else {
        "Show mode"
    };
    vec![
        span(app.show.name()),
        span(app.show.mark()).color(theme::PALETTE.warning),
        span(LINE),
        span(mode).color_maybe((!app.editable()).then_some(theme::PALETTE.primary)),
        span(LINE),
        span(format::count(count, "warning"))
            .color_maybe((count > 0).then_some(theme::PALETTE.warning)),
    ]
}

/// Returns the hints, each with the action whose keys it names when it can
/// give way
fn hints(app: &App) -> Vec<(Option<Action>, String)> {
    let keymap = &app.config.keymap;
    let key =
        |action: Action, label: &str| keymap.hint(action, label).map(|hint| (Some(action), hint));
    let typed = (!app.keys.is_empty()).then(|| (None, written(&app.keys)));
    let told = clipped(app)
        .or_else(|| app.told.clone())
        .or_else(|| notice(app))
        .map(|told| (None, told));
    let back = key(Action::Back, Action::Back.name()).filter(|_| app.can_go_back());
    let forward = key(Action::Forward, Action::Forward.name()).filter(|_| app.can_go_forward());
    let save = key(Action::Save, Action::Save.name()).filter(|_| app.show.unsaved());
    // The page covers an open menu, and takes its keys.  Enter picks
    // whichever key is bound to pick, so that it stands in where none is.
    let in_menu = app.menu && !app.help;
    let pick =
        key(Action::Pick, "pick").or_else(|| Some((Some(Action::Pick), "Enter: pick".to_owned())));
    let close = key(Action::CloseMenu, "close");
    typed
        .into_iter()
        .chain(told)
        .chain(pick.filter(|_| in_menu))
        .chain(close.filter(|_| in_menu))
        .chain(back)
        .chain(forward)
        .chain(save)
        .chain(key(Action::Go, "GO"))
        .chain(key(Action::Panic, "Panic"))
        .chain(key(Action::Help, Action::Help.name()))
        .collect()
}

/// Tells what the latest copy or cut took, with the key that pastes it and,
/// for a cut, the key that takes it back
fn clipped(app: &App) -> Option<String> {
    let keymap = &app.config.keymap;
    let clipped = app.clipped.as_ref()?;
    let paste = keymap.hint(Action::PasteBelow, "paste");
    let undo = keymap
        .hint(Action::Undo, Action::Undo.name())
        .filter(|_| clipped.cut && app.workspace.undoable().is_some());
    Some(
        iter::once(clipped.told.clone())
            .chain(paste)
            .chain(undo)
            .join(LINE),
    )
}

/// Tells the latest change to the show, with the key that takes it back,
/// or that makes it again after an undo, while that key can act
fn notice(app: &App) -> Option<String> {
    let workspace = &app.workspace;
    let (turn, change) = workspace.latest_change()?;
    let told = format::change(workspace, change);
    let (told, action, next) = match turn {
        Turn::Done => (capitalized(&told), Action::Undo, workspace.undoable()),
        Turn::Undone => (format!("Undid: {told}"), Action::Redo, workspace.redoable()),
        Turn::Redone => (format!("Redid: {told}"), Action::Undo, workspace.undoable()),
    };
    next.map(|_| answered(told, &app.config.keymap, action))
}

fn capitalized(text: &str) -> String {
    text.chars()
        .take(1)
        .flat_map(char::to_uppercase)
        .chain(text.chars().skip(1))
        .collect()
}
