// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading and writing presses as the configuration file writes them

use std::fmt;

use itertools::Itertools;
use keyboard_types::{Key, NamedKey};

use super::{MODIFIERS, Name, Press};

/// Name of the space bar, whose key value would read as the space between
/// two keys
const SPACE: &str = "Space";

/// Modifiers that a press can hold, in the order they are written
const HELD: [NamedKey; 3] = [NamedKey::Control, NamedKey::Alt, NamedKey::Shift];

impl Name {
    pub(super) fn read(name: &str) -> Option<Self> {
        match name {
            SPACE => Some(Name::Char(' ')),
            name => match name.parse::<Key>().ok()? {
                Key::Named(named) => Some(Name::Named(named)),
                Key::Character(text) => text.chars().exactly_one().ok().map(Name::Char),
            },
        }
    }
}

impl Press {
    fn read(token: &str) -> Result<Self, String> {
        let (held, key) = modifiers(token)?;
        let key = Name::read(key).ok_or_else(|| format!("`{token}` names no key"))?;
        let shift = held.contains(&NamedKey::Shift);
        // Shift on a letter makes the capital, which is the key it types
        let (key, shift) = match key {
            Name::Char(c) if shift && c.is_ascii_alphabetic() => {
                (Name::Char(c.to_ascii_uppercase()), false)
            }
            Name::Char(c) if shift && c != ' ' => {
                return Err(format!("`{token}` is written as the character it types"));
            }
            Name::Named(named) if MODIFIERS.contains(&named) => {
                return Err(format!("`{token}` only changes the keys pressed with it"));
            }
            key => (key, shift),
        };
        Ok(Self {
            key,
            ctrl: held.contains(&NamedKey::Control),
            alt: held.contains(&NamedKey::Alt),
            shift,
        })
    }
}

impl fmt::Display for Press {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        HELD.iter()
            .zip([self.ctrl, self.alt, self.shift])
            .filter(|(_, held)| *held)
            .try_for_each(|(modifier, _)| write!(f, "{modifier}+"))?;
        match self.key {
            Name::Char(' ') => f.write_str(SPACE),
            Name::Char(c) => write!(f, "{c}"),
            Name::Named(named) => write!(f, "{named}"),
        }
    }
}

pub fn sequence(text: &str) -> Result<Vec<Press>, String> {
    text.split_whitespace().map(Press::read).collect()
}

/// Writes presses as the configuration file does, apart by spaces
pub fn written(presses: &[Press]) -> String {
    presses.iter().join(" ")
}

/// Writes presses as the screen shows them: a sequence of keys that type a
/// character is joined, as Vim writes `dd`, so that nobody reads the
/// space of `d d` as a key to press, which would be GO.  A sequence with
/// any other key, which could not be told apart joined, keeps its spaces.
pub fn shown(presses: &[Press]) -> String {
    let separator = if presses.iter().all(|press| press.types_character()) {
        ""
    } else {
        " "
    };
    presses.iter().join(separator)
}

/// Splits the modifiers that a press holds off its key, as in `Control+d`.
/// A `+` with nothing after it is the key itself.
/// The modifiers are taken in a loop, since a token may repeat one without
/// end.
fn modifiers(token: &str) -> Result<(Vec<NamedKey>, &str), String> {
    let mut held = Vec::new();
    let mut rest = token;
    loop {
        match rest.split_once('+') {
            Some((name, key)) if !key.is_empty() => match name.parse::<NamedKey>() {
                Ok(modifier) if HELD.contains(&modifier) => {
                    held.push(modifier);
                    rest = key;
                }
                Ok(_) => {
                    return Err(format!(
                        "`{rest}` holds `{name}`, but only Control, Alt, and Shift can be held"
                    ));
                }
                Err(_) => return Ok((held, rest)),
            },
            _ => return Ok((held, rest)),
        }
    }
}
