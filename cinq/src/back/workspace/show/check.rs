// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What keeps a show file from opening: identities that cues or
//! instruments share, and levels and addresses outside their ranges

use itertools::Itertools;

use crate::back::{
    LEVELS, LOUDEST, PERCENT, SILENCE, Workspace,
    patch::{ADDRESSES, UNIVERSES, addressable},
};

/// Returns why the show read from a file cannot be opened, as
/// [`Workspace::from_show`] tells, or none when it can
pub(super) fn problem(workspace: &Workspace) -> Option<String> {
    let duplicate = workspace.cues().map(|cue| cue.id.0).duplicates().next();
    let unplayable = workspace
        .cues()
        .find(|cue| cue.kind.has_level() && !LEVELS.contains(&cue.level));
    let twin = workspace
        .patch
        .iter()
        .map(|instrument| instrument.id.0)
        .duplicates()
        .next();
    let unaddressable = workspace
        .patch
        .iter()
        .find(|instrument| !addressable(instrument.universe, instrument.address));
    let unlit = workspace
        .cues()
        .find(|cue| cue.lights.values().any(|level| !PERCENT.contains(level)));
    let problems = [
        duplicate.map(|id| format!("more than one cue has the id {id}")),
        unplayable.map(|cue| {
            format!(
                "the cue with the id {} has the level {} dB, which is not between {SILENCE} and \
                 {LOUDEST} dB",
                cue.id.0, cue.level
            )
        }),
        twin.map(|id| format!("more than one instrument has the id {id}")),
        unaddressable.map(|instrument| {
            format!(
                "the instrument with the id {} has the address {}/{}, which is not in a universe \
                 from {} to {} between {} and {}",
                instrument.id.0,
                instrument.universe,
                instrument.address,
                UNIVERSES.start(),
                UNIVERSES.end(),
                ADDRESSES.start(),
                ADDRESSES.end()
            )
        }),
        unlit.map(|cue| {
            format!(
                "the cue with the id {} has a light level that is not between 0 and 100 percent",
                cue.id.0
            )
        }),
    ];
    problems.into_iter().flatten().next()
}
