// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text boxes of a field, and the whole row of its target

use iced::{
    Element, Fill,
    widget::{row, text_input},
};

use super::{Field, locked, shown, target_menu, typed_file};
use crate::{
    back::{Cue, Edit, Kind},
    front::{
        App, Message, Setting,
        view::{FILE, NAME, SPACING, button, reached, theme},
    },
};

impl Field {
    /// Returns the text box of this field that holds `value` and applies the
    /// text typed into it as `edit`, or the value in a box that the keyboard
    /// skips, with the reason `ignored`, where the show cannot be edited or
    /// the cue ignores the value
    pub(super) fn entry<'a>(
        self,
        app: &App,
        cue: &Cue,
        ignored: Option<&'static str>,
        value: &str,
        edit: fn(String) -> Edit,
    ) -> Element<'a, Message> {
        let id = cue.id;
        let has_file = cue.kind.has_file();
        if !app.editable() || ignored.is_some() {
            return locked(value, self.placeholder(cue), ignored, Fill);
        }
        let input = text_input(self.placeholder(cue), value)
            .on_input(move |text| match self {
                // Path applies once the field is left, as a level does
                Field::Target if has_file => Message::SettingTyped(Setting::File(id), text),
                _ => Message::Edit(id, edit(text)),
            })
            .style(theme::field(self.valid(cue)));
        // Enter in the empty File field opens the chooser, since Tab skips
        // the Choose button, and leaves the field once a file is named
        reached(match self {
            Field::Name => input.id(NAME),
            Field::Target if has_file => input
                .id(FILE)
                .on_submit_maybe(value.is_empty().then_some(Message::ChooseFile(id))),
            _ => input,
        })
    }

    /// Returns what the Target field shows for the kind of `cue`: a file
    /// with a button that chooses one, a menu of the cues to target, the
    /// target as plain text, a destination, or nothing
    pub(super) fn target<'a>(
        self,
        app: &'a App,
        cue: &'a Cue,
        ignored: Option<&'static str>,
    ) -> Element<'a, Message> {
        let editable = app.editable();
        match cue.kind {
            Kind::Audio | Kind::Video | Kind::Picture => row![
                self.entry(app, cue, ignored, &typed_file(app, cue), Edit::File),
                button("Choose").on_press_maybe(editable.then_some(Message::ChooseFile(cue.id))),
            ]
            .spacing(SPACING)
            .into(),
            Kind::Fade | Kind::Start | Kind::Stop | Kind::Pause | Kind::Devamp | Kind::GoTo
                if editable =>
            {
                target_menu(app, cue)
            }
            Kind::Fade | Kind::Start | Kind::Stop | Kind::Pause | Kind::Devamp | Kind::GoTo => {
                shown(
                    &app.workspace
                        .target(cue)
                        .map_or_else(String::new, Cue::to_string),
                )
            }
            Kind::Network => self.entry(app, cue, ignored, &cue.destination, Edit::Destination),
            Kind::Light | Kind::Group | Kind::Wait | Kind::Memo => shown(""),
        }
    }
}
