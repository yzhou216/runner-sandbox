// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Why the text typed into a setting of an output does not apply, in words
//! that tell what the setting takes, for the line of help beside the field

use crate::{
    back::{Channels, DEVICE_CHANNELS, LOUDEST, Output, SILENCE, parse_level},
    front::{App, Setting},
};

/// Returns why `name` cannot be the name of `output`, if it cannot: an
/// empty name names nothing, and another output of the name would leave the
/// sends of cues ambiguous in show files
pub(super) fn name(app: &App, output: &Output, name: &str) -> Option<String> {
    if name.trim().is_empty() {
        return Some("the name is empty".to_owned());
    }
    app.workspace
        .outputs()
        .iter()
        .any(|other| other.id != output.id && other.name == name)
        .then(|| "another output has this name".to_owned())
}

/// Returns what an output takes for its channels, which the help beside the
/// field tells before anything is typed
pub(super) fn channels_help() -> String {
    format!("Channels 1 to {DEVICE_CHANNELS}: 3 for one, 1-2 or 1,3 for two")
}

/// Returns why `text` does not read as the channels of an output, if it does
/// not
pub(super) fn channels(text: &str) -> Option<String> {
    Channels::parse(text)
        .is_none()
        .then(|| format!("channels are 1 to {DEVICE_CHANNELS}: 3 for one, 1-2 or 1,3 for two"))
}

/// Returns why `text` is not a level for new cues to send at, if it is not.
/// An empty one is fine, since it sends nothing.
pub(super) fn level(text: &str) -> Option<String> {
    (!text.trim().is_empty() && parse_level(text).is_none())
        .then(|| format!("levels run from {SILENCE:+} to {LOUDEST:+} dB, or empty for none"))
}

/// Returns why `text` cannot be the value of `setting`, if it cannot, which
/// the status bar tells once the keyboard leaves a field that holds it
pub(in crate::front) fn rejection(app: &App, setting: Setting, text: &str) -> Option<String> {
    match setting {
        Setting::OutputName(id) => name(app, app.workspace.output(id)?, text),
        Setting::Channels(_) => channels(text),
        Setting::NewCues(_) | Setting::Send(..) => level(text),
    }
}
