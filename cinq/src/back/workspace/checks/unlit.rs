// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Light cues that set no instrument, or name one that the patch lacks
//!
//! Removing an instrument from the patch leaves its levels in the cues that
//! name it, so that undo brings them back, and a light cue that names no
//! instrument at all sets nothing.  GO still fires such a cue, since its
//! other instruments and its time count, so that it is a warning and no
//! problem.

use std::fmt;

use itertools::Itertools;

use crate::back::{Cue, InstrumentId, Kind, Workspace};

/// What keeps a light cue from setting all that it should
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unlit {
    Empty,
    /// Cue names these instruments, which the patch does not hold
    Unpatched(Vec<InstrumentId>),
}

impl fmt::Display for Unlit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("sets no instrument"),
            Self::Unpatched(ids) => {
                let noun = if ids.len() == 1 {
                    "instrument"
                } else {
                    "instruments"
                };
                let names = ids.iter().map(|id| id.0).join(", ");
                write!(f, "names {noun} {names}, which the patch lacks")
            }
        }
    }
}

impl Workspace {
    /// Returns what keeps the light cue `cue` from setting all that it
    /// names, or none for any other cue
    pub fn unlit(&self, cue: &Cue) -> Option<Unlit> {
        let missing: Vec<InstrumentId> = cue
            .lights
            .keys()
            .copied()
            .filter(|id| self.patch.iter().all(|instrument| instrument.id != *id))
            .collect();
        match cue.kind {
            Kind::Light if cue.lights.is_empty() => Some(Unlit::Empty),
            Kind::Light if !missing.is_empty() => Some(Unlit::Unpatched(missing)),
            _ => None,
        }
    }

    /// Returns the light cues that set no instrument or name one that the
    /// patch lacks, each with what is wrong
    pub fn unlit_cues(&self) -> impl Iterator<Item = (&Cue, Unlit)> {
        self.cues()
            .filter_map(|cue| self.unlit(cue).map(|unlit| (cue, unlit)))
    }
}
