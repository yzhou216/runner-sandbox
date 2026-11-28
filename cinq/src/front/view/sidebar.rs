// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue lists, running cues, warnings, and the audio device, with the
//! warnings above the audio device, which a show sets once, so that the first
//! warning shows whole at the least height of the window

mod devices;
mod problems;
mod running;
#[cfg(test)]
mod tests;

use iced::{
    Element, Fill,
    widget::{column, container, rule, scrollable, text},
};
use unicode_width::UnicodeWidthStr;

use super::{
    CURRENT_LIST, SIDEBAR, SIDEBAR_SCROLL, SPACING, char_width,
    cue_table::{RADIO, SCROLL_BAR},
    faults, radio,
    widget::shortened,
};
use crate::{
    back::one_line,
    front::{App, Message},
};

/// Least width of the sidebar, enough for the name of a short cue list, such
/// as `Act 1`, with its count beside the radio button, so that a narrow
/// window does not cut the names of the lists to the same two letters
const LEAST: f32 = 160.0;

/// Returns the width of the sidebar in a window `window` wide, which is a
/// quarter of it, so that a narrow window, such as one tiled beside another,
/// leaves the cue list room, but at least [`LEAST`] and at most [`SIDEBAR`]
pub(super) fn width(window: f32) -> f32 {
    (window / 4.0).clamp(LEAST, SIDEBAR)
}

pub(super) fn view(app: &App, width: f32) -> Element<'_, Message> {
    let columns = (width - 2.0 * SPACING - SCROLL_BAR) / char_width();
    scrollable(
        column![
            text("Cue lists"),
            lists(app, columns),
            rule::horizontal(1),
            outputs(app),
            rule::horizontal(1),
            text(format!("Active ({})", app.workspace.active().len())),
            // Empty list leaves out its body, as the header tells that it is empty
            active(app, columns),
            rule::horizontal(1),
            problems::header(app),
            problems::list(app),
            rule::horizontal(1),
            text("Default device"),
            devices::view(app),
        ]
        .spacing(SPACING)
        .padding(SPACING)
        .width(Fill),
    )
    .spacing(0)
    .id(SIDEBAR_SCROLL)
    .width(width)
    .height(Fill)
    .into()
}

fn lists(app: &App, columns: f32) -> Element<'_, Message> {
    // Each list takes one line, so that the rest of the sidebar stays in view,
    // and the inspector holds the whole name
    let room = columns - RADIO / char_width();
    let entries = app
        .workspace
        .lists()
        .iter()
        .enumerate()
        .map(|(index, list)| {
            let entry = radio(
                label(&one_line(&list.name), list.cues.len(), room as usize),
                index,
                (!app.outputs_shown).then_some(app.list),
                Message::SwitchList,
            );
            current(entry, !app.outputs_shown && index == app.list)
        });
    column(entries).spacing(SPACING).into()
}

fn outputs(app: &App) -> Element<'_, Message> {
    current(
        radio(
            format!("Outputs ({})", app.workspace.outputs().len()),
            (),
            app.outputs_shown.then_some(()),
            |()| Message::ShowOutputs,
        ),
        app.outputs_shown,
    )
}

fn active(app: &App, columns: f32) -> Option<Element<'_, Message>> {
    let workspace = &app.workspace;
    let failing = faults::failing(app);
    let entries = workspace
        .active()
        .iter()
        .filter_map(|active| {
            workspace.cue(active.id).map(|cue| {
                let silent = workspace.sends_to_failing(cue, &failing);
                running::active_cue(
                    active,
                    cue,
                    workspace.target(cue),
                    columns as usize,
                    silent,
                    running::idle(workspace, active, cue),
                )
            })
        })
        .collect::<Vec<_>>();
    (!entries.is_empty()).then(|| column(entries).spacing(SPACING).into())
}

/// Returns `entry`, marked for the sidebar to scroll to when it `shows`
fn current(entry: Element<'_, Message>, shows: bool) -> Element<'_, Message> {
    if shows {
        container(entry).id(CURRENT_LIST).into()
    } else {
        entry
    }
}

/// Returns the name of a list with the number of its cues, in `columns`
/// columns, where the name gives way to the number
fn label(name: &str, count: usize, columns: usize) -> String {
    let count = format!(" ({count})");
    format!(
        "{}{count}",
        shortened(name, columns.saturating_sub(count.width()))
    )
}
