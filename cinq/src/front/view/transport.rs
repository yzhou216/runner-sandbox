// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! GO button, standby display, and show-wide playback controls

use iced::{
    Element, Fill,
    widget::{checkbox, column, container, row, text, text::Wrapping},
};
use itertools::Itertools;

use super::{SPACING, button, format, line_height, theme};
use crate::front::{App, Message};

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let list = app.current();
    let standby = list.standby.and_then(|id| app.workspace.cue(id));
    // Summary keeps to three lines, so that the parts below stay where they are
    // as standby moves
    let lines = match standby {
        Some(cue) => {
            let notes = if cue.notes.trim().is_empty() {
                "none".to_owned()
            } else {
                cue.notes.lines().join(" ")
            };
            [
                format!("Standby: {cue}"),
                format!("Notes: {notes}"),
                format::chain(&app.workspace, list, cue),
            ]
        }
        None => [
            "Standby: end of list".to_owned(),
            "Notes: none".to_owned(),
            "GO fires nothing".to_owned(),
        ],
    };
    // Notes and chain each wrap to one line, which gives way at a word, so that
    // none is cut through a word beside the controls
    let [title, notes, chain] = lines.map(|line| text(line).wrapping(Wrapping::Word));
    let one_line = |line| container(line).height(line_height()).clip(true);
    let summary = column![
        title.size(24).wrapping(Wrapping::None),
        one_line(notes),
        one_line(chain),
    ];
    // GO still presses with nothing to fire, since it steps standby past cues that
    // cannot play, but it looks as a disabled button does
    let fires = app.workspace.next_go(list).next().is_some();
    let paused = app.workspace.active().iter().any(|active| active.paused);
    row![
        button(text("GO").size(40))
            .style(if fires {
                theme::go
            } else {
                theme::go_without_cues
            })
            .on_press_maybe(standby.map(|_| Message::Go)),
        container(summary).width(Fill).clip(true),
        column![
            row![
                button("Pause all").on_press_maybe(app.running().then_some(Message::PauseAll)),
                button("Resume all").on_press_maybe(paused.then_some(Message::ResumeAll)),
                button("Panic").style(theme::panic).on_press(Message::Panic),
            ]
            .spacing(SPACING),
            checkbox(app.workspace.show_mode())
                .label("Show mode")
                .on_toggle(Message::ShowMode),
        ]
        .spacing(SPACING),
    ]
    .spacing(SPACING)
    .padding(SPACING)
    .into()
}
