// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Notices of the status bar about clips, pastes, silences, stops, and undos

use super::{count, leading, number, short};
use crate::back::{Cue, Output};

/// Tells what a copy or cut took with `verb`, by the number of the cue, or
/// else by how many cues
pub fn clipped(verb: &str, cues: &[Cue]) -> String {
    of_cues(verb, cues.iter())
}

/// Tells what `verb` did to `cues`, by the number of the cue, or else by how
/// many cues
pub(super) fn of_cues<'a>(verb: &str, mut cues: impl ExactSizeIterator<Item = &'a Cue>) -> String {
    match (cues.len(), cues.next()) {
        (1, Some(cue)) => format!("{verb} cue {}", number(cue)),
        (count, _) => format!("{verb} {count} cues"),
    }
}

/// Tells what a paste of `cues` cues left behind, `sends` sends to outputs
/// and `targets` targets that the show has no namesake of, and how many of
/// them are `unpatched` light cues, or nothing where it left none
pub fn pasted_losses(
    cues: usize,
    sends: usize,
    targets: usize,
    unpatched: usize,
) -> Option<String> {
    let lost = [(sends, "send"), (targets, "target")]
        .into_iter()
        .filter(|&(number, _)| number > 0)
        .map(|(number, noun)| count(number, noun))
        .collect::<Vec<_>>();
    let told = [
        (!lost.is_empty()).then(|| format!("dropped {}", lost.join(" and "))),
        (unpatched > 0).then(|| format!("{unpatched} unpatched")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    (!told.is_empty()).then(|| format!("Pasted {}, {}", count(cues, "cue"), told.join(", ")))
}

/// Returns whether `told`, the notice of a change, says that a cue plays
/// nowhere or sends to no output now, which the operator cannot see or hear,
/// where an undo of it brings the sound back
pub fn silences(told: &str) -> bool {
    ["to play nowhere", "to no output"]
        .iter()
        .any(|phrase| told.contains(phrase))
        && !told.to_lowercase().starts_with("undid")
}

/// Tells after `told`, which names what an edit did to `cut` cues, that
/// `stopped` of them were running and stopped by it, since the silence has no
/// other cause to read from the bar
pub fn stopping(told: String, cut: usize, stopped: usize) -> String {
    match (cut, stopped) {
        (_, 0) => told,
        (1, _) => format!("{told}, which stopped"),
        (_, stopped) => format!("{told}, {stopped} of which stopped"),
    }
}

/// Tells what an undo or a redo, by `verb`, took back or made again, which is
/// `turns` changes, the last of them being `told`, or that one change alone
pub fn turned(verb: &str, turns: usize, told: &str) -> String {
    match turns {
        0 | 1 => format!("{verb}: {told}"),
        turns => format!("{verb} {turns} changes, the last: {told}"),
    }
}

/// Tells what a copy or cut of outputs took with `verb`, in lower case, by the
/// name of the output, or else by how many outputs, after `facts` of what it
/// did to the cues, which leave out the words that name the output as one
pub fn clipped_outputs(verb: &str, outputs: &[Output], facts: Option<String>) -> String {
    let told = match outputs {
        [output] => format!("{verb} {}", the_output(&output.name, facts.is_some())),
        outputs => format!("{verb} {} outputs", outputs.len()),
    };
    leading(facts, &told)
}

/// Names an output as "the output Sub", or as "Sub" where `terse`, which
/// leaves room for what a change did to the cues
pub(super) fn the_output(name: &str, terse: bool) -> String {
    if terse {
        short(name)
    } else {
        format!("the output {}", short(name))
    }
}
