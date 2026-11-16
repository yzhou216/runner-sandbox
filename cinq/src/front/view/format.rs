// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text for back-end values, kept apart from the widgets that show it

mod consequences;
#[cfg(test)]
mod tests;

use std::path::Path;

use itertools::Itertools;

use crate::back::{
    Active, Advance, Change, Cue, CueList, Edit, Kind, Media, Output, OutputId, Phase, Workspace,
    moved_output,
};
pub use consequences::{consequences, consequences_of, leading, short};

/// Formats seconds as `m:ss.s`
fn clock(seconds: f64) -> String {
    let tenths = (seconds.max(0.0) * 10.0).round() as u32;
    format!("{}:{:02}.{}", tenths / 600, tenths / 10 % 60, tenths % 10)
}

/// Leaves zero blank, so that real waits stand out
pub fn wait(seconds: f64) -> String {
    if seconds > 0.0 {
        clock(seconds)
    } else {
        String::new()
    }
}

/// Formats the length of a cue's action, or a hold for a picture cue, whose
/// action lasts until something stops it
pub fn action(cue: &Cue) -> String {
    let length = clock(cue.action().into());
    match (cue.kind, &cue.media) {
        (Kind::Audio | Kind::Video, Media::Loaded { .. }) if cue.loops() => {
            format!("{length} loop")
        }
        (Kind::Audio | Kind::Video, Media::Loaded { .. }) | (Kind::Fade | Kind::Wait, _) => length,
        (Kind::Picture, Media::Loaded { .. }) => "hold".to_owned(),
        (Kind::Audio | Kind::Video | Kind::Picture, _) => "?".to_owned(),
        (Kind::Stop | Kind::Memo, _) => "-".to_owned(),
    }
}

/// Leaves the post-wait blank for auto-follow, which ignores it
pub fn post_wait(cue: &Cue) -> String {
    if cue.advance == Advance::Follow {
        String::new()
    } else {
        wait(cue.post_wait.value().into())
    }
}

/// Returns the short label of the Then column
pub fn advance(advance: Advance) -> &'static str {
    match advance {
        Advance::Manual => "",
        Advance::Continue => "continue",
        Advance::Follow => "follow",
    }
}

pub fn target(workspace: &Workspace, cue: &Cue) -> String {
    match cue.kind {
        Kind::Audio | Kind::Video | Kind::Picture => cue
            .file_name()
            .and_then(|name| Path::new(name).file_name())
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
        Kind::Fade | Kind::Stop => workspace.target(cue).map_or_else(String::new, number),
        Kind::Wait | Kind::Memo => String::new(),
    }
}

/// Formats a level in dB without the unit, as the level field holds it
pub fn decibels(db: f32) -> String {
    format!("{db:+.1}")
}

/// Counts things by name, as in `1 cue` and `2 cues`
pub fn count(number: usize, noun: &str) -> String {
    let plural = if number == 1 { "" } else { "s" };
    format!("{number} {noun}{plural}")
}

pub fn phase(active: &Active) -> String {
    match active.phase() {
        Phase::Scheduled { starts_in: wait } | Phase::PreWait { remaining: wait } => {
            format!("in {wait:.1}s")
        }
        Phase::Running { remaining } => format!("{} left", clock(remaining)),
        Phase::Looping => "looping".to_owned(),
        Phase::Showing => "showing".to_owned(),
    }
}

/// Describes an active cue in full for the list of running cues
pub fn activity(active: &Active, cue: &Cue) -> String {
    let paused = if active.paused { "paused, " } else { "" };
    let fading = active.fade.map_or("", |_| ", fading");
    let level = if cue.sounds() {
        format!(", {} dB", decibels(active.level))
    } else {
        String::new()
    };
    format!("{paused}{}{fading}{level}", phase(active))
}

/// Describes every cue of the chain that the next GO starts by its number,
/// with how long after GO each cue after the first fires, so that
/// auto-continue and auto-follow chains are visible before they run.  The
/// count leaves out the cues that cannot play, which GO skips, and each of
/// them says why.  A chain with no cue that can play says so in words
/// before it names the cues and why each cannot play.
pub fn chain(workspace: &Workspace, list: &CueList, standby: &Cue) -> String {
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
        let delay = (index > 0).then(|| format!("+{delay:.1}s"));
        let skipped = workspace
            .problem(cue)
            .map(|problem| format!("skipped: {problem}"));
        let notes = delay.into_iter().chain(skipped).join(", ");
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

/// Describes a change to the show in few words, starting in lower case, as
/// the status bar tells what undo or redo acts on
pub fn change(workspace: &Workspace, change: &Change) -> String {
    let named = |id| workspace.cue(id).map_or_else(String::new, number);
    match change {
        Change::Added { cues, .. } => match cues.as_slice() {
            [placed] => format!("added cue {}", number(&placed.cue)),
            cues => format!("added {} cues", cues.len()),
        },
        Change::Pasted { cues, .. } => match cues.as_slice() {
            [placed] => format!("pasted cue {}", number(&placed.cue)),
            cues => format!("pasted {} cues", cues.len()),
        },
        Change::Deleted { cues, cut, .. } => {
            let verb = if *cut { "cut" } else { "deleted" };
            match cues.as_slice() {
                [placed] => format!("{verb} cue {}", number(&placed.cue)),
                cues => format!("{verb} {} cues", cues.len()),
            }
        }
        Change::Moved { cue, .. } => format!("moved cue {}", named(*cue)),
        Change::Edited { cue, redo, .. } => {
            let cue = named(*cue);
            match redo {
                Edit::Number(_) => format!("changed the number of cue {cue}"),
                Edit::Name(_) => format!("changed the name of cue {cue}"),
                Edit::File(_) => format!("changed the file of cue {cue}"),
                Edit::FileAndName { .. } => format!("chose the file of cue {cue}"),
                Edit::Start(_) => format!("changed the start of cue {cue}"),
                Edit::End(_) => format!("changed the end of cue {cue}"),
                Edit::Loop(_) => format!("changed whether cue {cue} loops"),
                Edit::Target(_) => format!("changed the target of cue {cue}"),
                Edit::Notes(_) => format!("changed the notes of cue {cue}"),
                Edit::PreWait(_) => format!("changed the pre-wait of cue {cue}"),
                Edit::Duration(_) => format!("changed the duration of cue {cue}"),
                Edit::PostWait(_) => format!("changed the post-wait of cue {cue}"),
                Edit::Advance(_) => format!("changed what follows cue {cue}"),
                Edit::Level(_) => format!("changed the level of cue {cue}"),
                Edit::StopTarget(_) => format!("changed whether cue {cue} stops its target"),
                Edit::Sends(_) => format!("changed where cue {cue} plays"),
            }
        }
        Change::Standby(standby) => match standby.after.and_then(|id| workspace.cue(id)) {
            Some(cue) => format!("moved standby to cue {}", number(cue)),
            None => "moved standby past the end".to_owned(),
        },
        Change::Restored { .. } => "brought back the unsaved changes".to_owned(),
        Change::Outputs {
            before,
            after,
            cut,
            pasted,
            anchor,
            ..
        } => outputs(workspace, before, after, (*cut, *pasted), *anchor),
    }
}

/// Tells what a copy or cut took with `verb`, by the number of the cue, or
/// else by how many cues
pub fn clipped(verb: &str, cues: &[Cue]) -> String {
    match cues {
        [cue] => format!("{verb} cue {}", number(cue)),
        cues => format!("{verb} {} cues", cues.len()),
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
fn the_output(name: &str, terse: bool) -> String {
    if terse {
        short(name)
    } else {
        format!("the output {}", short(name))
    }
}

/// Describes a change to the outputs by what differs between them before
/// and after it, leaving the words that name an output as one where the
/// change did something to the cues, which [`consequences`] tells first
fn outputs(
    workspace: &Workspace,
    before: &[Output],
    after: &[Output],
    (cut, pasted): (bool, bool),
    selected: Option<OutputId>,
) -> String {
    let terse = consequences(workspace, before, after).is_some();
    let the = |name: &str| the_output(name, terse);
    let removed = if cut { "cut" } else { "deleted" };
    let added = if pasted { "pasted" } else { "added" };
    let pairs = || {
        before.iter().filter_map(|was| {
            let now = after.iter().find(|now| now.id == was.id)?;
            Some((was, now))
        })
    };
    let edited = pairs().find_map(|(was, now)| {
        let name = &short(&now.name);
        if was.name != now.name {
            Some(format!("renamed the output {} to {name}", short(&was.name)))
        } else if was.device != now.device {
            Some(format!("changed the device of the output {name}"))
        } else if was.channels != now.channels {
            Some(format!("changed the channels of the output {name}"))
        } else if was.new_cues != now.new_cues {
            Some(format!("changed what new cues send to the output {name}"))
        } else {
            None
        }
    });
    let moved =
        || moved_output(before, after, selected).map(|now| format!("moved {}", the(&now.name)));
    match (
        missing(after, before).as_slice(),
        missing(before, after).as_slice(),
    ) {
        ([output], []) => Some(format!("{added} {}", the(&output.name))),
        ([], [output]) => Some(format!("{removed} {}", the(&output.name))),
        (put, []) if !put.is_empty() => Some(format!("{added} {} outputs", put.len())),
        ([], gone) if !gone.is_empty() => Some(format!("{removed} {} outputs", gone.len())),
        (put, deleted) if !put.is_empty() || !deleted.is_empty() => {
            Some("changed the outputs".to_owned())
        }
        _ => edited.or_else(moved),
    }
    .unwrap_or_else(|| "changed the outputs".to_owned())
}

pub fn number(cue: &Cue) -> String {
    if cue.number.trim().is_empty() {
        format!("\"{}\"", cue.name)
    } else {
        cue.number.clone()
    }
}

/// Returns the outputs of `from` that `of` lacks
fn missing<'a>(from: &'a [Output], of: &[Output]) -> Vec<&'a Output> {
    from.iter()
        .filter(|output| of.iter().all(|other| other.id != output.id))
        .collect()
}
