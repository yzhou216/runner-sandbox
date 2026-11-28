// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text for the chain that the next GO fires

use itertools::Itertools;

use super::{count, elsewhere, number, span};
use crate::back::{Cue, CueList, Kind, OutputId, Workspace};

/// Describes every cue of the chain that the next GO starts by its number,
/// with how long after GO each cue after the first fires, so that
/// auto-continue and auto-follow chains are visible before they run.  The
/// count leaves out the cues that cannot play, which GO skips, and each of
/// them says why.  A cue that sends to no output that can play, among `failing`
/// or in the show, is told as silent, though GO fires it.  A chain with no cue
/// that can play says so before it names the cues and why each cannot.
pub fn chain(workspace: &Workspace, list: &CueList, standby: &Cue, failing: &[OutputId]) -> String {
    let fired = workspace.next_go(list).count();
    let cues = list.chain(standby.id);
    if fired == 0 {
        let reasons = cues.filter_map(|(cue, _)| {
            let problem = workspace.problem(cue)?;
            Some(format!("cue {} ({problem})", number(cue)))
        });
        return format!("GO fires nothing: {}", reasons.format(", "));
    }
    let described = cues.enumerate().map(|(index, (cue, delay))| {
        let delay = (index > 0).then(|| format!("+{}", span(delay.into())));
        let skipped = workspace
            .problem(cue)
            .map(|problem| format!("skipped: {problem}"));
        let notes = delay
            .into_iter()
            .chain(acts(workspace, cue))
            .chain(skipped)
            .chain(silence(workspace, cue, failing))
            .join(", ");
        if notes.is_empty() {
            number(cue)
        } else {
            format!("{} ({notes})", number(cue))
        }
    });
    format!(
        "GO fires {}: {}",
        count(fired, "cue"),
        described.format(", ")
    )
}

/// Tells that `cue` makes no sound, for want of an output or because the
/// outputs that it sends to are among `failing`, or nothing where it can play
fn silence(workspace: &Workspace, cue: &Cue, failing: &[OutputId]) -> Option<String> {
    let why = if workspace.sends_nowhere(cue) {
        "no output"
    } else if workspace.sends_to_failing(cue, failing) {
        "output failed"
    } else {
        return None;
    };
    Some(format!("silent: {why}"))
}

/// Tells that a `GoTo` cue is about to move standby to `target` in `seconds`,
/// for the chain to lead with, since the standby that the panel names is
/// about to change
pub fn moving(target: &Cue, seconds: f64) -> String {
    format!("Standby moves to {} in {}", number(target), span(seconds))
}

/// Tells how many cues are paused, for the chain to lead with, since a GO
/// fires beside them
pub fn paused(count: usize) -> Option<String> {
    (count > 0).then(|| format!("{count} paused"))
}

/// Tells what the control cue `cue` does, as `GoTo 2` or
/// `start E3 in Effects`, which is none for any other cue and for one that has
/// no target
fn acts(workspace: &Workspace, cue: &Cue) -> Option<String> {
    let verb = match cue.kind {
        Kind::Start => "start",
        Kind::Stop => "stop",
        Kind::Pause => "pause",
        Kind::Devamp => "devamp",
        Kind::GoTo => "GoTo",
        _ => return None,
    };
    let target = workspace.target(cue)?;
    Some(format!(
        "{verb} {}{}",
        number(target),
        elsewhere(workspace, cue, target)
    ))
}
