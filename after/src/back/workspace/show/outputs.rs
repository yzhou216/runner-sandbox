// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Outputs as a show file holds them, and the sends of cues by the names
//! of the outputs
//!
//! A file without outputs has Main alone, and a show with Main alone, as it
//! was made, writes none, so that a file written before outputs reads as
//! it did.  Outputs take their identities from their order in the file.

use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::back::{Channels, Device, LEVELS, LOUDEST, Output, OutputId, SILENCE, Sends};

/// Output as the file names it.  An output without a device plays on the
/// default device, and its channels count from one, as people count them.
#[derive(Serialize, Deserialize)]
pub(super) struct OutputFile {
    name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    device: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    device_name: String,
    channels: String,
    new_cues: Option<f32>,
}

/// Returns the outputs that `files` hold, or why they cannot be read: two
/// outputs of one name, channels that are neither one nor two, or a level
/// that no cue could send at
pub(super) fn read(files: Vec<OutputFile>) -> Result<Vec<Output>, String> {
    if files.is_empty() {
        return Ok(vec![Output::main(OutputId(1))]);
    }
    if let Some(name) = files
        .iter()
        .map(|file| file.name.as_str())
        .duplicates()
        .next()
    {
        return Err(format!("more than one output has the name {name}"));
    }
    files
        .into_iter()
        .zip(1..)
        .map(|(file, id)| file.output(OutputId(id)))
        .collect()
}

/// Returns the outputs as the file holds them, none for Main alone
pub(super) fn written(outputs: &[Output]) -> Vec<OutputFile> {
    if outputs == [Output::main(OutputId(1))] {
        return Vec::new();
    }
    outputs.iter().map(OutputFile::of).collect()
}

/// Returns `sends`, by the names of the outputs, as `outputs` read them
pub(super) fn sends_by_id(
    sends: BTreeMap<String, f32>,
    outputs: &[Output],
    cue: u32,
) -> Result<Sends, String> {
    let ids: HashMap<&str, OutputId> = outputs
        .iter()
        .map(|output| (output.name.as_str(), output.id))
        .collect();
    sends
        .into_iter()
        .map(|(name, level)| {
            let id = ids.get(name.as_str()).ok_or_else(|| {
                format!("the cue with the id {cue} sends to {name}, which no output is named")
            })?;
            playable(level, || {
                format!("the cue with the id {cue} sends to {name}")
            })?;
            Ok((*id, level))
        })
        .collect()
}

/// Returns `sends` by the names of the outputs, leaving out those to
/// outputs that have gone
pub(super) fn sends_by_name(sends: &Sends, outputs: &[Output]) -> BTreeMap<String, f32> {
    outputs
        .iter()
        .filter_map(|output| Some((output.name.clone(), *sends.get(&output.id)?)))
        .collect()
}

/// Returns why no cue could play at `level`, as one typed wrong by hand,
/// such as 60 for -60, would play at a thousand times the amplitude of its
/// file
fn playable(level: f32, what: impl Fn() -> String) -> Result<(), String> {
    if LEVELS.contains(&level) {
        Ok(())
    } else {
        Err(format!(
            "{} at {level} dB, which is not between {SILENCE} and {LOUDEST} dB",
            what()
        ))
    }
}

impl OutputFile {
    fn output(self, id: OutputId) -> Result<Output, String> {
        let channels = Channels::parse(&self.channels).ok_or_else(|| {
            format!(
                "the output {} plays on the channels {}, which are neither one channel nor two, \
                 counted from 1",
                self.name, self.channels
            )
        })?;
        if let Some(level) = self.new_cues {
            playable(level, || {
                format!("new cues send to the output {}", self.name)
            })?;
        }
        let device = if self.device.is_empty() {
            Device::Default
        } else {
            Device::Named {
                id: self.device,
                name: self.device_name,
            }
        };
        Ok(Output {
            id,
            name: self.name,
            device,
            channels,
            new_cues: self.new_cues,
        })
    }

    fn of(output: &Output) -> Self {
        let (device, device_name) = match &output.device {
            Device::Default => (String::new(), String::new()),
            Device::Named { id, name } => (id.clone(), name.clone()),
        };
        Self {
            name: output.name.clone(),
            device,
            device_name,
            channels: output.channels.to_string(),
            new_cues: output.new_cues,
        }
    }
}
