// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Help page, which lists every action that has a key and fills the window
//!
//! The keys of the next and previous cue scroll it, and those of the first
//! and last cue go to its top and its bottom.

#[cfg(test)]
mod tests;

use std::ffi::OsString;

use iced::{
    Element, Fill,
    widget::{
        column, container, rich_text, scrollable, span, table, text,
        text::{IntoFragment, Span},
    },
};
use strum::VariantArray;

use super::{HELP, SPACING, button, theme};
use crate::front::{
    App, Message,
    keymap::{Action, Keymap, config::Source},
};

/// Widest that the text of the page grows, so that its lines stay easy to
/// read in a window that the page fills
const WIDTH: f32 = 960.0;

/// Space above and below the text of each row of the table, closer than in
/// the cue list, so that the page fits the window without scrolling
const ROW_GAP: f32 = 2.0;

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
    let keymap = &app.config.keymap;
    let bindings = table::table(
        [
            table::column(text("Keys"), |action: Action| text(keymap.listed(action))),
            table::column(text("Action"), |action: Action| text(action.description())).width(Fill),
            table::column(text("Name in config.toml"), |action: Action| {
                text(action.name())
            }),
        ],
        bound(keymap),
    )
    .padding_y(ROW_GAP);
    let operators = Some(keymap.operator_keys())
        .filter(|keys| !keys.is_empty())
        .map(|keys| text(format!("After {}, a key that selects the next, previous, first, or last cue or output applies the action to the cues or outputs from the selection to where that key would move it.  A count before the operator, before the key, or both counts the move, and the two multiply.", keys.join(" or "))));
    let key_values = paragraph(
        "The manual page cinq(5) describes config.toml, where keys go by their W3C UI Events key values, listed at ",
        link(KEY_VALUES, OsString::from(KEY_VALUES)),
        ", apart from the space bar, which is Space.",
        Message::OpenLink,
    );
    let page = column![
        text(format!("Keys of the {} keymap", keymap.name())).size(24),
        text(first_steps(keymap)),
        bindings,
        text(counts(keymap)),
    ]
    .extend(operators.map(Element::from))
    .push(text("Actions that this keymap binds to no key are left out of the table, and cinq(5) lists them all."))
    .push(text("The keys of a sequence are typed one after another with no Space between them, since Space is GO."))
    .push(text("The keys of GO and Panic act at once, even halfway through another command."))
    .push(source(&app.config.source))
    .push(key_values)
    .extend(app.link_problem.as_deref().map(|why| text(why).into()))
    .extend(notice())
    .push(button("Close").on_press(Message::CloseHelp))
    .spacing(SPACING)
    .width(Fill)
    .max_width(WIDTH);
    let scrolling = scrollable(page)
        .id(HELP)
        .spacing(0)
        .width(Fill)
        .height(Fill);
    container(scrolling).padding(SPACING).into()
}

/// Returns the actions that the keymap binds to a key, in the order of the
/// page, since a row with no key would only crowd the page
fn bound(keymap: &Keymap) -> Vec<Action> {
    Action::VARIANTS
        .iter()
        .copied()
        .filter(|&action| keymap.key(action).is_some())
        .collect()
}

fn source(source: &Source) -> Element<'_, Message> {
    let (before, file, after) = match source {
        Source::File(file) => ("These keys come from ", file, ", and change as it does."),
        Source::Missing(file) => (
            "These are the default keys, which a file at ",
            file,
            " can change.  Following the link makes it.",
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
    rich_text([span(before), link, span(after)])
        .on_link_click(open)
        .into()
}

fn link<'a, Link>(label: impl IntoFragment<'a>, target: Link) -> Span<'a, Link> {
    span(label).link(target).color(theme::LINK).underline(true)
}

/// Tells in a few lines what to press first, with the keys of the keymap in
/// use, since the table below puts the keys of scrolling before those that
/// add and name a cue
fn first_steps(keymap: &Keymap) -> String {
    let key = |action| keymap.key(action).unwrap_or_else(|| "(no key)".to_owned());
    format!(
        "First steps: {go} is GO, which fires the standby cue, and {panic} is Panic, which stops everything.  {next} and {previous} move the selection, and {standby} puts the selected cue on standby.  {add} adds a cue below the selection, {name} names it, and {undo} undoes the last change.  The table lists every key, and cinq(5) describes the rest.",
        go = key(Action::Go),
        panic = key(Action::Panic),
        next = key(Action::Next),
        previous = key(Action::Previous),
        standby = key(Action::Standby),
        add = key(Action::AddBelow),
        name = key(Action::ChangeName),
        undo = key(Action::Undo),
    )
}

/// Tells what a count typed before a key does, which is the same for the
/// cues and the outputs that show in their place
fn counts(keymap: &Keymap) -> String {
    let example = keymap
        .key(Action::Next)
        .map_or_else(String::new, |keys| format!(", as in 3 {keys}"));
    format!(
        "A count typed before a key repeats a move{example}.  It goes that many places back or forward, undoes or redoes that many changes, copies, cuts, deletes, or adds that many cues or outputs, pastes that many times, goes that many matches along, scrolls that many rows, and goes that many pages down or up.  It picks the row for the first and last cue and for putting a row in place, the row in view for the top and bottom rows, and the tab for the next tab, and sets the rows of a half page."
    )
}
