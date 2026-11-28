// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Settings of the output that the outputs view has selected
//!
//! A name that another output has, channels that are neither one nor two,
//! and a level out of range stay in their fields, marked, without applying,
//! as a level of a cue does.  The line beside each field, in dim text, tells
//! what the setting takes.  It turns red and tells why where the value is not
//! applied, so that the color is never the only sign.

mod device;
mod settings;
#[cfg(test)]
mod tests;

use iced::{
    Center, Element, Fill,
    widget::{Text, column, container, row, scrollable, text, text_input},
};

use super::{lock, locked::locked};
use crate::{
    back::{Output, one_line},
    front::{
        App, Message, Setting,
        message::reasons::rejection,
        view::{FORMS, SPACING, char_width, dropdown_width, reached, theme},
    },
};

/// Columns of the font that the labels of the settings take
const LABEL: f32 = 14.0;

/// Columns of the font that the name field holds, with room for a long
/// name of a speaker
const NAME_ROOM: f32 = 24.0;

/// Columns of the font that the channels field holds, as many as `256,256`
/// takes, with one to spare
const CHANNELS_ROOM: f32 = 8.0;

/// Columns of the font that the device list shows of a name, as many as
/// `Dante Virtual Soundcard, not found` takes
const DEVICE_ROOM: f32 = 34.0;

pub(super) fn room(columns: f32) -> f32 {
    char_width() * columns + text_input::DEFAULT_PADDING.x()
}

/// Returns the field of `setting` that holds `columns` columns of text, with
/// the text that it shows: what was typed into it, or else `value`
pub(super) fn typing<'a>(
    app: &App,
    setting: Setting,
    placeholder: &str,
    value: String,
    columns: f32,
) -> (text_input::TextInput<'a, Message>, String) {
    let shown = match &app.typed {
        Some((typed, text)) if *typed == setting => text.clone(),
        _ => value,
    };
    let input = text_input(placeholder, &shown)
        .on_input_maybe(
            app.editable()
                .then_some(move |text| Message::SettingTyped(setting, text)),
        )
        .width(room(columns));
    (input, shown)
}

/// Returns `input` as `typing` made it, in the style of whether `valid`, while
/// the show can be edited, or else `shown` in a box that the keyboard skips
pub(super) fn styled<'a>(
    app: &App,
    input: text_input::TextInput<'a, Message>,
    shown: &str,
    placeholder: &str,
    columns: f32,
    valid: bool,
) -> Element<'a, Message> {
    if app.editable() {
        reached(input.style(theme::field(valid)))
    } else {
        locked(shown, placeholder, None, room(columns).into())
    }
}

/// Returns the reason that a value is not applied, in red
pub(super) fn refused<'a>(reason: &str) -> Text<'a> {
    text(format!("Not applied: {reason}")).style(text::danger)
}

/// Returns the reason that the text typed into the latest field is not
/// applied, if the field is of a setting that `of` accepts and it is not
pub(super) fn refusal(app: &App, of: impl Fn(&Setting) -> bool) -> Option<String> {
    app.typed
        .as_ref()
        .filter(|(setting, _)| of(setting))
        .and_then(|(setting, typed)| rejection(app, *setting, typed))
}

/// Returns the line of help beside a setting: `hint`, in dim text, or the
/// reason that the value is not applied, in red, where there is one
pub(super) fn help<'a>(hint: impl Into<String>, rejection: Option<String>) -> Element<'a, Message> {
    match rejection {
        Some(reason) => refused(&reason).width(Fill).into(),
        None => text(hint.into()).style(text::secondary).width(Fill).into(),
    }
}

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let Some(output) = app.output.and_then(|id| app.workspace.output(id)) else {
        return text("Select an output to inspect it").into();
    };
    // Scrolls, so that no field is cut to a sliver in a low or narrow window
    // and Tab can reach one below the part in view
    scrollable(
        column![text(format!(
            "Output: {}{}",
            one_line(&output.name),
            lock(app)
        ))]
        .extend(settings::of(app, output).into_iter().map(setting))
        .push(sending(app, output))
        .spacing(SPACING)
        .width(Fill),
    )
    .spacing(SPACING)
    .id(FORMS)
    .width(Fill)
    .height(Fill)
    .into()
}

fn setting<'a>((label, cells): (&'static str, [Element<'a, Message>; 2])) -> Element<'a, Message> {
    row![text(label).width(char_width() * LABEL)]
        .extend(cells)
        .spacing(SPACING)
        .align_y(Center)
        .into()
}

fn sending<'a>(app: &App, output: &Output) -> Text<'a> {
    let sending = app
        .workspace
        .cues()
        .filter(|cue| cue.kind.plays_file() && app.workspace.sends(cue).contains_key(&output.id))
        .count();
    text(match sending {
        1 => "1 cue sends here".to_owned(),
        count => format!("{count} cues send here"),
    })
}

/// Returns the device list, as wide as the name that it holds and no wider
fn device<'a>(app: &'a App, output: &'a Output) -> Element<'a, Message> {
    container(device::view(app, output))
        .max_width(dropdown_width(DEVICE_ROOM))
        .into()
}
