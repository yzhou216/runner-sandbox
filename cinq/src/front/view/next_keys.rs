// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keys that can follow those typed so far, which a box lists while a
//! command waits for more, as which-key does in Emacs and as Helix does

use iced::{
    Element, Fill, never,
    widget::{column, container, rich_text, span},
};
use itertools::Itertools;
use unicode_width::UnicodeWidthStr;

use super::{SPACING, theme};
use crate::front::{App, Message, keymap::written};

#[cfg(test)]
mod tests;

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
            span(format!("{keys}{padding}")).color(theme::PALETTE.primary),
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
/// its keys, as the help page lists them, and its description
fn listed(app: &App) -> Vec<(String, String)> {
    app.config
        .keymap
        .continuations(&app.keys)
        .chunk_by(|&(_, action, operator)| (action, operator))
        .into_iter()
        .map(|((action, operator), group)| {
            let description = operator.map_or_else(
                || action.description().to_owned(),
                |operator| operator.over(action),
            );
            (
                group.map(|(keys, ..)| written(keys)).join(", "),
                description,
            )
        })
        .collect()
}
