// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue as a show file holds it, with the settings that its kind uses

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::outputs;
use crate::back::{Advance, Cue, CueId, InstrumentId, Kind, Output};

/// Setting that is `None` stays out of the file, since TOML has no null,
/// and one that the file leaves out reads as `None`.
#[derive(Serialize, Deserialize)]
pub(super) struct CueFile {
    id: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    number: String,
    kind: Kind,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    file: String,
    start: Option<f32>,
    end: Option<f32>,
    #[serde(default, rename = "loop", skip_serializing_if = "std::ops::Not::not")]
    looping: bool,
    loop_start: Option<f32>,
    loop_end: Option<f32>,
    #[serde(default, skip_serializing_if = "is_top")]
    depth: u8,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    collapsed: bool,
    target: Option<u32>,
    pre_wait: Option<f32>,
    duration: Option<f32>,
    post_wait: Option<f32>,
    #[serde(default, skip_serializing_if = "is_manual")]
    advance: Advance,
    level: Option<f32>,
    stop_target: Option<bool>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    notes: String,
    /// Levels at which the cue sends to outputs, by their names
    sends: Option<BTreeMap<String, f32>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    lights: BTreeMap<InstrumentId, f32>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    destination: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    message: String,
}

impl CueFile {
    pub(super) fn of(cue: &Cue, file: String, outputs: &[Output]) -> Self {
        let set = |seconds: f32| Some(seconds).filter(|&seconds| seconds > 0.0);
        let plays = cue.kind.plays_file();
        let network_only = |text: &str| {
            if cue.kind == Kind::Network {
                text.to_owned()
            } else {
                String::new()
            }
        };
        Self {
            id: cue.id.0,
            number: cue.number.clone(),
            kind: cue.kind,
            name: cue.name.clone(),
            file,
            start: set(cue.start.value()).filter(|_| plays),
            end: cue.end.time().filter(|_| plays),
            looping: plays && cue.looping,
            loop_start: cue.loop_start.time().filter(|_| plays),
            loop_end: cue.loop_end.time().filter(|_| plays),
            depth: cue.depth,
            collapsed: cue.kind == Kind::Group && cue.collapsed,
            target: cue
                .target
                .filter(|_| cue.kind.has_target())
                .map(|target| target.0),
            pre_wait: set(cue.pre_wait.value()),
            duration: cue.kind.has_duration().then(|| cue.duration.value()),
            post_wait: set(cue.post_wait.value()),
            advance: cue.advance,
            level: cue.kind.has_level().then_some(cue.level),
            stop_target: (cue.kind == Kind::Fade).then_some(cue.stop_target),
            notes: cue.notes.clone(),
            sends: cue
                .sends
                .as_ref()
                .filter(|_| plays)
                .map(|sends| outputs::sends_by_name(sends, outputs)),
            lights: if cue.kind == Kind::Light {
                cue.lights.clone()
            } else {
                BTreeMap::new()
            },
            destination: network_only(&cue.destination),
            message: network_only(&cue.message),
        }
    }

    pub(super) fn cue(self, outputs: &[Output]) -> Result<Cue, String> {
        let sends = self
            .sends
            .map(|sends| outputs::sends_by_id(sends, outputs, self.id))
            .transpose()?;
        Ok(Cue::new(CueId(self.id), self.number, self.kind, self.name)
            .file(self.file)
            .maybe_start(self.start)
            .maybe_end(self.end)
            .looping(self.looping)
            .maybe_loop_start(self.loop_start)
            .maybe_loop_end(self.loop_end)
            .depth(self.depth)
            .collapsed(self.kind == Kind::Group && self.collapsed)
            .maybe_target(self.target.map(CueId))
            .maybe_pre_wait(self.pre_wait)
            .maybe_duration(self.duration)
            .maybe_post_wait(self.post_wait)
            .advance(self.advance)
            .maybe_level(self.level)
            .maybe_stop_target(self.stop_target)
            .notes(self.notes)
            .maybe_sends(sends)
            .lights(self.lights)
            .destination(self.destination)
            .message(self.message)
            .build())
    }
}

fn is_top(depth: &u8) -> bool {
    *depth == 0
}

fn is_manual(advance: &Advance) -> bool {
    *advance == Advance::Manual
}
