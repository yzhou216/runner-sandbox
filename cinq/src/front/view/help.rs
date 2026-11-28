// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Help page, which lists every action that has a key and fills the window
//!
//! The keys of the next and previous cue scroll it, and those of the first
//! and last cue go to its top and its bottom.

#[cfg(test)]
mod tests;
mod texts;

use std::ffi::OsString;

use iced::{
    Element, Fill,
    widget::{
        column, container, responsive, rich_text, scrollable, span, table, text,
        text::{IntoFragment, Span},
    },
};
use texts::{counts, first_steps, operators};
use unicode_width::UnicodeWidthStr;

use super::{
    HELP, SPACING, button, char_width,
    cue_table::{CELL_SPACE, SCROLL_BAR},
    hinted, theme, widest,
};
use crate::front::{
    App, Message,
    config::Source,
    keymap::{Action, Keymap},
};

/// Widest that the paragraphs of the page grow, so that their lines stay easy to
/// read in a window that the page fills
const WIDTH: f32 = 960.0;

/// Widest that the table of keys grows, which is wider than the paragraphs
/// since its descriptions wrap in half of that, but not so wide that a key
/// lies far from its name in a window that the page fills
const TABLE_WIDTH: f32 = 1.5 * WIDTH;

/// Space above and below the text of each row of the table, closer than in
/// the cue list, so that the page fits the window without scrolling
const ROW_GAP: f32 = 2.0;

/// Columns that the descriptions keep beside the keys and the names in the
/// configuration file, so that most of them wrap to two lines or fewer
const DESCRIPTION_ROOM: f32 = 40.0;

const KEY_VALUES: &str = "https://www.w3.org/TR/uievents-key/";

const LICENSES: &str = "https://www.gnu.org/licenses/";

/// Notice of the license of cinq, as the README gives it, whose last
/// paragraph goes on with a link to [`LICENSES`]
const NOTICE: [&str; 3] = [
    "cinq is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.",
    "This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.",
    "You should have received a copy of the GNU General Public License along with this program.  If not, see ",
];

pub(super) fn view(app: &App) -> Element<'_, Message> {
    responsive(move |window| page(app, window.width)).into()
}

/// Returns the table of the actions that have a key, with the names that the
/// configuration file gives them where `names` is set
fn bindings(keymap: &Keymap, names: bool) -> Element<'_, Message> {
    let keys = table::column(text("Keys"), |action: Action| text(keymap.listed(action)));
    let actions =
        table::column(text("Action"), |action: Action| text(action.description())).width(Fill);
    let named = table::column(text("Name in config.toml"), |action: Action| {
        text(action.name())
    });
    table::table(
        [keys, actions].into_iter().chain(names.then_some(named)),
        bound(keymap),
    )
    .padding_y(ROW_GAP)
    .into()
}

/// Returns whether a table in `room` pixels keeps its column of names in
/// the configuration file, which it does while the descriptions keep
/// [`DESCRIPTION_ROOM`] columns beside it and the keys, and cinq(5) lists the
/// names for any window that is too narrow
fn names_fit(keymap: &Keymap, room: f32) -> bool {
    let widest = |title: &str, text: &dyn Fn(Action) -> String| {
        widest(bound(keymap).into_iter().map(text)).max(title.width())
    };
    let columns = widest("Keys", &|action| keymap.listed(action))
        + widest("Name in config.toml", &|action| action.name().to_owned());
    (columns as f32 + DESCRIPTION_ROOM) * char_width() + 3.0 * CELL_SPACE <= room
}

fn page(app: &App, window: f32) -> Element<'_, Message> {
    let keymap = &app.config.keymap;
    let room = (window - 2.0 * SPACING - SCROLL_BAR).min(TABLE_WIDTH);
    let bindings = container(bindings(keymap, names_fit(keymap, room)))
        .width(Fill)
        .max_width(TABLE_WIDTH);
    let page = column![prose(top(keymap)), bindings, prose(bottom(app))].spacing(SPACING);
    let scrolling = scrollable(page)
        .id(HELP)
        .spacing(0)
        .width(Fill)
        .height(Fill);
    // Below the page rather than on it, so that it is in view however far
    // the page has scrolled from the link
    let problem = app
        .problems
        .link
        .as_deref()
        .map(|why| container(text(why)).max_width(WIDTH).into());
    let parts = column![scrolling].extend(problem).spacing(SPACING);
    container(parts).padding(SPACING).into()
}

/// Returns the title of the page, what to press first, and the link to the
/// tutorial, above the table
fn top(keymap: &Keymap) -> Element<'_, Message> {
    let tutorial = paragraph(
        "The ",
        link("tutorial", ()),
        " shows step by step how to run a show and how to make one.",
        |()| Message::OpenTutorial,
    );
    column![
        text(format!("Keys of the {} keymap", keymap.name())).size(24),
        text(first_steps(keymap)),
        tutorial,
    ]
    .spacing(SPACING)
    .into()
}

/// Returns the notes below the table, from the counts to the notice of the
/// license and the button that closes the page
fn bottom(app: &App) -> Element<'_, Message> {
    let keymap = &app.config.keymap;
    let key_values = paragraph(
        "The manual page cinq(5) describes config.toml, where keys go by their W3C UI Events key values, listed at ",
        link(KEY_VALUES, OsString::from(KEY_VALUES)),
        ", apart from the space bar, which is Space.",
        Message::OpenLink,
    );
    column![text(counts(keymap))]
        .extend(operators(keymap).map(|operators| Element::from(text(operators))))
        .push(text("Actions that this keymap binds to no key are left out of the table, and cinq(5) lists them all."))
        .extend(keymap.has_sequences().then(|| text("The keys of a sequence are typed one after another with no Space between them, since Space is GO.").into()))
        .push(text("The keys of GO and Panic act at once, even halfway through another command."))
        .push(source(&app.config.source))
        .push(key_values)
        .extend(notice())
        .push(button("Close").on_press(Message::CloseHelp))
        .spacing(SPACING)
        .into()
}

fn prose<'a>(part: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(part).width(Fill).max_width(WIDTH).into()
}

/// Returns the actions that the keymap binds to a key, in the order of the
/// page, since a row with no key would only crowd the page
fn bound(keymap: &Keymap) -> Vec<Action> {
    Action::listed()
        .filter(|&action| keymap.key(action).is_some())
        .collect()
}

fn source(source: &Source) -> Element<'_, Message> {
    let (before, file, after) = match source {
        Source::File(file) => ("These keys come from ", file, ", and change as it does."),
        Source::Missing(file) => (
            "These are the default keys, which a file at ",
            file,
            " can change.  Following the link creates it.",
        ),
        Source::NoDirectory => return text("These are the default keys.").into(),
    };
    paragraph(
        before,
        link(file.display().to_string(), file.clone()),
        after,
        Message::EditConfig,
    )
}

fn notice<'a>() -> [Element<'a, Message>; 3] {
    let [free, warranty, copy] = NOTICE;
    let licenses = link(LICENSES, OsString::from(LICENSES));
    [
        text(free).into(),
        text(warranty).into(),
        paragraph(copy, licenses, ".", Message::OpenLink),
    ]
}

fn paragraph<'a, Link: Clone + 'static>(
    before: &'a str,
    link: Span<'a, Link>,
    after: &'a str,
    open: impl Fn(Link) -> Message + 'a,
) -> Element<'a, Message> {
    let hint = link.link.clone().map(&open);
    let text = rich_text([span(before), link, span(after)]).on_link_click(open);
    match hint {
        Some(message) => hinted(text, message),
        None => text.into(),
    }
}

fn link<'a, Link>(label: impl IntoFragment<'a>, target: Link) -> Span<'a, Link> {
    span(label).link(target).color(theme::LINK).underline(true)
}
