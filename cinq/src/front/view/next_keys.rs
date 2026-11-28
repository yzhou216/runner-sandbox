// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keys that can follow those typed so far, which a box lists while a
//! command waits for more, as which-key does in Emacs and as Helix does

#[cfg(test)]
mod tests;

use iced::{
    Element, Fill, never,
    widget::{column, container, rich_text, span},
};
use itertools::Itertools;
use unicode_width::UnicodeWidthStr;

use super::{SPACING, theme};
use crate::front::{
    App, Message,
    keymap::{Rows, shown},
};

pub(super) fn view(app: &App) -> Option<Element<'_, Message>> {
    if app.help {
        return None;
    }
    let next = listed(app);
    let width = next.iter().map(|(keys, _)| keys.width()).max()?;
    // Monospace font lines the descriptions up after the padded keys, which
    // are padded by the columns they take, as a wide character takes two
    let lines = next.into_iter().map(|(keys, description)| {
        let padding = " ".repeat(width - keys.width());
        rich_text([
            span(format!("{keys}{padding}")).color(theme::BLUE_COOLER),
            span(format!("  {description}")),
        ])
        .on_link_click(never)
        .into()
    });
    let list = container(column(lines))
        .style(container::bordered_box)
        .padding(SPACING);
    Some(
        container(list)
            .align_right(Fill)
            .align_bottom(Fill)
            .padding(SPACING)
            .into(),
    )
}

/// Returns what can follow the keys typed so far, each action once with all
/// its keys, as the help page lists them, and its description.  The line of
/// the key typed again, such as `g` after `g`, comes first, since it is the
/// one that the doubled binding most often means.
fn listed(app: &App) -> Vec<(String, String)> {
    let rows = if app.outputs_shown {
        Rows::Outputs
    } else {
        Rows::Cues
    };
    let count = app.config.keymap.pending_count(&app.keys);
    app.config
        .keymap
        .continuations(&app.keys)
        // Keys that show mode or the stage window refuse are not taught, as in the status bar
        .filter(|&(_, action, operator)| app.refusal(operator.unwrap_or(action)).is_none())
        .chunk_by(|&(_, action, operator)| (action, operator))
        .into_iter()
        .map(|((action, operator), group)| {
            let description = operator.map_or_else(
                || action.description_counted(rows, count),
                |operator| operator.over(action, rows, count),
            );
            let group: Vec<_> = group.collect();
            let repeats = operator.is_none()
                && group
                    .iter()
                    .any(|&(keys, ..)| matches!(keys, [key] if app.keys.last() == Some(key)));
            let keys = group.into_iter().map(|(keys, ..)| shown(keys)).join(", ");
            (!repeats, (keys, description))
        })
        .sorted_by_key(|&(later, _)| later)
        .map(|(_, line)| line)
        .collect()
}
