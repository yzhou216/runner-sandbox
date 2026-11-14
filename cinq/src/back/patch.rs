// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Light patch, which gives each instrument of a show the universe and the
//! DMX address of its intensity

use std::{fmt, ops::RangeInclusive};

use serde::{Deserialize, Serialize};

use super::items::replaced_where;

/// Universes as sACN numbers them, from 1, which Art-Net sends as its
/// Port-Addresses from 0
pub(super) const UNIVERSES: RangeInclusive<u16> = 1..=32_768;

pub(super) const ADDRESSES: RangeInclusive<u16> = 1..=512;

/// Identity of an instrument that stays stable when the operator renames
/// it, by which light cues name it
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstrumentId(pub u32);

/// Light of the patch, whose intensity one DMX address sets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instrument {
    pub id: InstrumentId,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default = "first", skip_serializing_if = "is_first")]
    pub universe: u16,
    pub address: u16,
}

/// Change that the operator makes to the patch
#[derive(Debug, Clone, PartialEq)]
pub enum PatchEdit {
    /// Adds an instrument named after its identity, the next after the
    /// highest, at the address after that of the last instrument, as QLab
    /// numbers new instruments
    Add,
    Remove(InstrumentId),
    Name(InstrumentId, String),
    /// Universe and DMX address of an instrument
    Address(InstrumentId, u16, u16),
}

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.name.trim().is_empty() {
            write!(f, "{}", self.id.0)
        } else {
            f.write_str(&self.name)
        }
    }
}

impl Instrument {
    /// Returns the address as the patch shows it: the DMX address alone in
    /// universe 1, and after its universe and a slash in the others, as in
    /// `2/12`
    pub fn address_text(&self) -> String {
        match self.universe {
            1 => self.address.to_string(),
            universe => format!("{universe}/{}", self.address),
        }
    }
}

/// Returns the universe and the DMX address that `text` reads as, in the
/// form that [`Instrument::address_text`] gives
pub fn parse_address(text: &str) -> Option<(u16, u16)> {
    let (universe, address) = text.split_once('/').unwrap_or(("1", text));
    let number = |part: &str| part.trim().parse::<u16>().ok();
    Some((number(universe)?, number(address)?))
        .filter(|&(universe, address)| addressable(universe, address))
}

pub(super) fn addressable(universe: u16, address: u16) -> bool {
    UNIVERSES.contains(&universe) && ADDRESSES.contains(&address)
}

impl PatchEdit {
    /// Returns whether `next` edits the same setting of the same instrument,
    /// so that the two make one change
    pub(super) fn same_setting(&self, next: &PatchEdit) -> bool {
        match (self, next) {
            (Self::Name(id, _), Self::Name(next, _))
            | (Self::Address(id, ..), Self::Address(next, ..)) => id == next,
            _ => false,
        }
    }

    /// Returns `patch` with the edit made, where an instrument added takes
    /// the identity after the highest of `taken`, or as it was where the
    /// edit names no instrument of it, gives an address that is none, or
    /// adds an instrument once the identities have run out
    pub(super) fn applied(
        self,
        patch: Vec<Instrument>,
        taken: impl Iterator<Item = InstrumentId>,
    ) -> Vec<Instrument> {
        match self {
            Self::Add => {
                let next = taken.map(|id| u64::from(id.0) + 1).max().unwrap_or(1);
                let Ok(id) = u32::try_from(next) else {
                    return patch;
                };
                let (universe, address) = patch.last().map_or((first(), 1), |last| {
                    if last.address < *ADDRESSES.end() {
                        (last.universe, last.address + 1)
                    } else {
                        ((last.universe + 1).min(*UNIVERSES.end()), 1)
                    }
                });
                let added = Instrument {
                    id: InstrumentId(id),
                    name: id.to_string(),
                    universe,
                    address,
                };
                patch.into_iter().chain([added]).collect()
            }
            Self::Remove(id) => patch
                .into_iter()
                .filter(|instrument| instrument.id != id)
                .collect(),
            Self::Name(id, name) => replaced_where(
                patch,
                |instrument| instrument.id == id,
                |instrument| Instrument { name, ..instrument },
            ),
            Self::Address(id, universe, address) if addressable(universe, address) => {
                replaced_where(
                    patch,
                    |instrument| instrument.id == id,
                    |instrument| Instrument {
                        universe,
                        address,
                        ..instrument
                    },
                )
            }
            Self::Address(..) => patch,
        }
    }
}

fn first() -> u16 {
    *UNIVERSES.start()
}

fn is_first(universe: &u16) -> bool {
    *universe == first()
}
