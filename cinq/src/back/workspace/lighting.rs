// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rules of lights: the light patch, and the levels of its instruments,
//! which light cues fade and which stay once their cue has ended
//!
//! As in QLab, a light cue sets only the instruments that it names, so that
//! the others keep the levels that earlier cues gave them, and it takes
//! each instrument it names over from an older light cue that is still
//! fading it.  A fade follows the progress of its cue, so that it pauses
//! with the cue, and stopping the cue, as a panic does, leaves its
//! instruments where the fade has brought them.

use std::collections::BTreeMap;

use super::{Change, Workspace};
use crate::back::{Active, CueId, Instrument, InstrumentId, PatchEdit};

/// DMX channels of a universe
pub type Universe = [u8; 512];

/// Level of an instrument in percent, with the fade of the light cue that
/// moves it
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Dimmer {
    level: f32,
    fade: Option<Glide>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Glide {
    run: u32,
    from: f32,
    to: f32,
}

impl Dimmer {
    /// Returns the dimmer moved as far as the run of its fade has played,
    /// or left where it is once the run has ended
    fn followed(self, active: &[Active]) -> Self {
        let Some(glide) = self.fade else {
            return self;
        };
        let progress = active
            .iter()
            .find(|active| active.run == glide.run)
            .map(Active::progress);
        match progress {
            Some(progress) if progress < 1.0 => Self {
                level: glide.from + (glide.to - glide.from) * progress,
                ..self
            },
            Some(_) => Self {
                level: glide.to,
                fade: None,
            },
            None => Self { fade: None, ..self },
        }
    }
}

impl Workspace {
    pub fn patch(&self) -> &[Instrument] {
        &self.patch
    }

    /// Returns the level of an instrument in percent, which is none until a
    /// light cue sets it
    pub fn light_level(&self, id: InstrumentId) -> f32 {
        self.dimmers.get(&id).map_or(0.0, |dimmer| dimmer.level)
    }

    /// Returns the DMX channels of each universe that the patch uses, where
    /// instruments that share an address set it to the highest of their
    /// levels
    pub fn universes(&self) -> BTreeMap<u16, Universe> {
        self.patch
            .iter()
            .fold(BTreeMap::new(), |mut universes, instrument| {
                let channels = universes.entry(instrument.universe).or_insert([0; 512]);
                let channel = &mut channels[usize::from(instrument.address) - 1];
                *channel = (*channel).max(dmx(self.light_level(instrument.id)));
                universes
            })
    }

    /// Edits the patch and records the edit, unless it leaves the patch as
    /// it was.  An instrument added takes no identity that a light cue
    /// names, so that the levels of an instrument removed, which undo
    /// brings back, never pass to another.
    pub(super) fn patched(self, edit: PatchEdit) -> Self {
        let before = self.patch.clone();
        let named = self.cues().flat_map(|cue| cue.lights.keys().copied());
        let taken = before.iter().map(|instrument| instrument.id).chain(named);
        let after = edit.clone().applied(before.clone(), taken);
        if after == before {
            return self;
        }
        self.with_patch(after.clone()).recorded(Change::Patched {
            edit,
            before,
            after,
        })
    }

    pub(super) fn with_patch(self, patch: Vec<Instrument>) -> Self {
        Self { patch, ..self }
    }

    /// Starts the fades of the light cue of run `run` from the levels that
    /// its instruments have now
    pub(super) fn lit(self, id: CueId, run: u32) -> Self {
        let Some(cue) = self.cue(id) else {
            return self;
        };
        let fades: Vec<(InstrumentId, Dimmer)> = cue
            .lights
            .iter()
            .map(|(&instrument, &to)| {
                let from = self.light_level(instrument);
                let glide = Glide { run, from, to };
                (
                    instrument,
                    Dimmer {
                        level: from,
                        fade: Some(glide),
                    },
                )
            })
            .collect();
        let dimmers = self.dimmers.into_iter().chain(fades).collect();
        Self { dimmers, ..self }
    }

    /// Moves every instrument along the fade of the light cue that moves it
    pub(super) fn dimmed(self) -> Self {
        let dimmers = self
            .dimmers
            .iter()
            .map(|(&id, dimmer)| (id, dimmer.followed(&self.active)))
            .collect();
        Self { dimmers, ..self }
    }
}

/// Returns the DMX value of a level in percent
fn dmx(level: f32) -> u8 {
    (level.clamp(0.0, 100.0) * 2.55).round() as u8
}
