// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Light patch, which gives each instrument the universe and the DMX address
//! of its intensity

use std::{fmt, ops::RangeInclusive};

use serde::{Deserialize, Serialize};

use super::{cue::one_line, items::replaced_where};

/// Universes as sACN numbers them, from 1, which Art-Net sends as
/// Port-Addresses from 0
pub(super) const UNIVERSES: RangeInclusive<u16> = 1..=32_768;

/// DMX channels of a universe
pub(super) const SLOTS: usize = 512;

pub(super) const DARK: [u8; SLOTS] = [0; SLOTS];

/// Addresses that an instrument may have in its universe
pub(super) const ADDRESSES: RangeInclusive<u16> = 1..=SLOTS as u16;

/// Identity of an instrument that stays stable when it is renamed, by which
/// light cues name it
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

#[derive(Debug, Clone, PartialEq)]
pub enum PatchEdit {
    /// Adds an instrument named after its identity, the next after the
    /// highest, at the address after the last instrument, as QLab does
    Add,
    Remove(InstrumentId),
    Name(InstrumentId, String),
    Address(InstrumentId, u16, u16),
}

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if one_line(&self.name).is_empty() {
            write!(f, "{}", self.id.0)
        } else {
            f.write_str(&one_line(&self.name))
        }
    }
}

impl Instrument {
    /// Returns the address alone in universe 1, and after its universe and a
    /// slash in the others, as in `2/12`
    pub fn address_text(&self) -> String {
        match self.universe {
            1 => self.address.to_string(),
            universe => format!("{universe}/{}", self.address),
        }
    }
}

/// Reads the form of [`Instrument::address_text`]
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
    pub(super) fn same_setting(&self, next: &PatchEdit) -> bool {
        match (self, next) {
            (Self::Name(id, _), Self::Name(next, _))
            | (Self::Address(id, ..), Self::Address(next, ..)) => id == next,
            _ => false,
        }
    }

    /// An instrument added takes the identity after the highest of `taken`.
    /// The patch stays as it was where the edit names no instrument of it,
    /// gives an invalid address, or adds one once the identities or the
    /// addresses have run out.
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
                let Some((universe, address)) = patch.last().map_or(Some((first(), 1)), after)
                else {
                    return patch;
                };
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

/// Returns the address after that of `last`, which rolls over into the next
/// universe, or none after the last address of the last universe, where
/// another instrument would share an address
fn after(last: &Instrument) -> Option<(u16, u16)> {
    if last.address < *ADDRESSES.end() {
        Some((last.universe, last.address + 1))
    } else {
        (last.universe < *UNIVERSES.end()).then_some((last.universe + 1, 1))
    }
}

fn first() -> u16 {
    *UNIVERSES.start()
}

fn is_first(universe: &u16) -> bool {
    *universe == first()
}
