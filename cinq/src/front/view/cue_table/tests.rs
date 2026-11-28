// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the State column of the cue list and the columns that fit

mod collapsed;
mod fitting;
mod groups;
mod output_failed;
mod send_failed;
mod state;

use iced::{Theme, widget::text};

use super::Column;
use crate::{
    back::{Command, Cue, CueId, Edit, Kind, OutputId, Sends, Workspace, fresh, loaded, sound},
    front::view::theme,
};

fn show(sends: Option<Sends>) -> Workspace {
    let cue = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .file("rain.ogg")
        .build();
    fresh(vec![cue])
        .apply(loaded(1, "rain.ogg", sound(10.0)))
        .apply(Command::add_output(1, 1))
        .apply(Command::Edit(CueId(1), Edit::Sends(sends)))
        .apply(Command::DeleteOutputs(vec![OutputId(2)]))
}

fn texts(workspace: &Workspace, column: Column) -> Vec<String> {
    let groups = workspace.lists()[0].groups();
    groups
        .list()
        .cues
        .iter()
        .map(|cue| column.text(workspace, &groups, cue))
        .collect()
}

fn told<Style>(lines: Vec<(String, Style)>) -> Vec<String> {
    lines.into_iter().map(|(line, _)| line).collect()
}

/// Returns whether `style` is the color of a warning
fn yellow(style: impl Fn(&Theme) -> text::Style) -> bool {
    let theme = theme::modus_vivendi();
    style(&theme).color == text::warning(&theme).color
}

fn yellows<Style: Fn(&Theme) -> text::Style>(lines: Vec<(String, Style)>) -> Vec<(String, bool)> {
    lines
        .into_iter()
        .map(|(line, style)| (line, yellow(style)))
        .collect()
}
