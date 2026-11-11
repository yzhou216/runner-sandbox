// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Changes to the configuration file that cinq makes itself, which touch
//! only the line of their setting, so that the rest of the text stays as
//! the operator wrote it, comments and all

#[cfg(test)]
mod tests;

use std::{fs, ops::Range, path::Path};

use toml::{Value, de::DeTable};

use super::make;

const AUDIO_DEVICE: &str = "audio_device";

/// Keeps the audio device whose ID is `device` in `file`, making the file
/// when it is not there yet, or takes the setting out for the default
/// device
pub fn keep_audio_device(file: &Path, device: Option<&str>) -> Result<(), String> {
    let cannot = |why: String| format!("config.toml cannot keep the audio device: {why}");
    make(file).map_err(|error| cannot(error.to_string()))?;
    let text = fs::read_to_string(file).map_err(|error| cannot(error.to_string()))?;
    let changed = with_audio_device(&text, device).map_err(cannot)?;
    if changed == text {
        return Ok(());
    }
    fs::write(file, changed).map_err(|error| cannot(error.to_string()))
}

/// Returns `text` with `audio_device` set to `device`, or without it for
/// the default device.  A new setting goes before the first line that is
/// neither blank nor a comment, where it cannot fall into a table.
pub(super) fn with_audio_device(text: &str, device: Option<&str>) -> Result<String, String> {
    let table = DeTable::parse(text)
        .map_err(|_| "it is not TOML".to_owned())?
        .into_inner();
    let found = table
        .iter()
        .find(|(key, _)| key.get_ref() == AUDIO_DEVICE)
        .map(|(key, value)| (key.span(), value.span()));
    let quoted = device.map(|id| Value::String(id.to_owned()).to_string());
    Ok(match (found, quoted) {
        (Some((_, value)), Some(quoted)) => spliced(text, value, &quoted),
        (Some((key, value)), None) => spliced(text, lines(text, key.start..value.end), ""),
        (None, Some(quoted)) => {
            let line = format!("{AUDIO_DEVICE} = {quoted}\n");
            match first_setting(text) {
                Some(start) => spliced(text, start..start, &line),
                None if text.is_empty() || text.ends_with('\n') => format!("{text}{line}"),
                None => format!("{text}\n{line}"),
            }
        }
        (None, None) => text.to_owned(),
    })
}

fn spliced(text: &str, range: Range<usize>, with: &str) -> String {
    [&text[..range.start], with, &text[range.end..]].concat()
}

/// Returns the whole lines that `range` lies on, with the line break that
/// ends the last of them
fn lines(text: &str, range: Range<usize>) -> Range<usize> {
    let start = text[..range.start].rfind('\n').map_or(0, |line| line + 1);
    let end = text[range.end..]
        .find('\n')
        .map_or(text.len(), |line| range.end + line + 1);
    start..end
}

/// Returns where the first line that is neither blank nor a comment starts
fn first_setting(text: &str) -> Option<usize> {
    text.split_inclusive('\n')
        .scan(0, |start, line| {
            let at = *start;
            *start += line.len();
            Some((at, line.trim_start()))
        })
        .find(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .map(|(at, _)| at)
}
