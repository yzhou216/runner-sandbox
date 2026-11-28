// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What an edit of the light patch says of itself

use super::short;
use crate::back::{Instrument, InstrumentId, PatchEdit};

/// Describes the edit `redo` of the light patch that turned `before` into
/// `after`, by the instrument that it added, removed, renamed, or moved
pub(super) fn patched(redo: &PatchEdit, before: &[Instrument], after: &[Instrument]) -> String {
    let named = |patch: &[Instrument], id: InstrumentId| {
        patch
            .iter()
            .find(|instrument| instrument.id == id)
            .map_or_else(String::new, |instrument| short(&instrument.to_string()))
    };
    match redo {
        PatchEdit::Add => {
            let added = after
                .iter()
                .find(|instrument| before.iter().all(|was| was.id != instrument.id));
            added.map_or_else(
                || "added an instrument".to_owned(),
                |instrument| format!("added instrument {}", short(&instrument.to_string())),
            )
        }
        PatchEdit::Remove(id) => format!("removed instrument {}", named(before, *id)),
        PatchEdit::Name(id, _) => format!(
            "renamed instrument {} to {}",
            named(before, *id),
            named(after, *id)
        ),
        PatchEdit::Address(id, ..) => {
            let address = after
                .iter()
                .find(|instrument| instrument.id == *id)
                .map_or_else(String::new, Instrument::address_text);
            format!(
                "changed the address of instrument {} to {address}",
                named(after, *id)
            )
        }
    }
}
