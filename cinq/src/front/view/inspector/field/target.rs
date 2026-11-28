// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Menu of the cues that the target of a cue can be

use iced::Element;

use crate::{
    back::{Cue, Edit},
    front::{App, Message, view::dropdown},
};

pub(super) fn target_menu<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    let id = cue.id;
    let targets: Vec<&Cue> = app
        .workspace
        .cues()
        .filter(|other| cue.can_target(other))
        .collect();
    let selected = targets
        .iter()
        .find(|target| Some(target.id) == cue.target)
        .copied();
    let placeholder = target_placeholder(targets.is_empty());
    dropdown(
        targets,
        selected,
        move |target: &Cue| Message::Edit(id, Edit::Target(Some(target.id))),
        &app.config.keymap,
    )
    .placeholder(placeholder)
    .into()
}

/// Returns what the target menu says while nothing is chosen, which is not
/// an invitation to choose where the list holds no cue to choose
pub(super) fn target_placeholder(none_to_choose: bool) -> &'static str {
    if none_to_choose {
        "no cue to choose"
    } else {
        "choose a cue"
    }
}
