// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text for back-end values, kept apart from the widgets that show it.  The
//! update shares it, to word the notices that it puts in the status bar.

mod chain;
mod change;
mod consequences;
mod notices;
#[cfg(test)]
mod tests;

use std::path::Path;

use crate::back::{Active, Advance, Cue, Groups, Kind, Media, Phase, SILENCE, Workspace, one_line};
pub use chain::{chain, moving, paused};
pub use change::change;
use consequences::short;
pub use consequences::{consequences, consequences_of, leading};
pub use notices::{clipped, clipped_outputs, pasted_losses, silences, stopping, turned};
use notices::{of_cues, the_output};

/// Formats seconds as `m:ss.s`
fn clock(seconds: f64) -> String {
    let tenths = (seconds.max(0.0) * 10.0).round() as u64;
    format!("{}:{:02}.{}", tenths / 600, tenths / 10 % 60, tenths % 10)
}

/// Formats a time ahead as `4.5s`, or as a clock from a minute on
fn span(seconds: f64) -> String {
    if seconds < 59.95 {
        format!("{seconds:.1}s")
    } else {
        clock(seconds)
    }
}

/// Tells the loop that a cue plays where it differs from the times typed, as
/// when a time lies outside the clip, and nothing where the cue plays what was
/// typed
pub(super) fn loop_played(cue: &Cue) -> Option<String> {
    cue.loop_played()
        .map(|(start, end)| format!("Loop plays from {} to {}", clock(start), clock(end)))
}

/// Leaves zero blank, so that real waits stand out
pub(super) fn wait(seconds: f64) -> String {
    if seconds > 0.0 {
        clock(seconds)
    } else {
        String::new()
    }
}

/// Formats the length of a cue's action, or a hold for a picture cue, whose
/// action lasts until something stops it.  The action of a group lasts until
/// the last of those of the cues that it holds ends, which a cue that never
/// ends makes a hold, and is none for a group that holds no cue.
pub(super) fn action(groups: &Groups, cue: &Cue) -> String {
    let length = clock(groups.action_of(cue).into());
    match (cue.kind, &cue.media) {
        (Kind::Audio | Kind::Video, Media::Loaded { .. }) if cue.loops() => {
            format!("{length} loop")
        }
        (Kind::Audio | Kind::Video, Media::Loaded { .. })
        | (Kind::Light | Kind::Fade | Kind::Wait, _) => length,
        (Kind::Picture, Media::Loaded { .. }) => "hold".to_owned(),
        (Kind::Audio | Kind::Video | Kind::Picture, _) => "?".to_owned(),
        (
            Kind::Start
            | Kind::Stop
            | Kind::Pause
            | Kind::Devamp
            | Kind::GoTo
            | Kind::Network
            | Kind::Memo,
            _,
        ) => "-".to_owned(),
        (Kind::Group, _) if groups.endless(cue) => "hold".to_owned(),
        (Kind::Group, _) if groups.members(cue.id).is_empty() => "-".to_owned(),
        (Kind::Group, _) => length,
    }
}

/// Leaves the post-wait blank for auto-follow, which ignores it
pub(super) fn post_wait(cue: &Cue) -> String {
    if cue.advance == Advance::Follow {
        String::new()
    } else {
        wait(cue.post_wait.value().into())
    }
}

pub(super) fn advance(advance: Advance) -> &'static str {
    match advance {
        Advance::Manual => "",
        Advance::Continue => "continue",
        Advance::Follow => "follow",
    }
}

/// Formats what a cue acts on: its file, its target, or, for a group, how
/// many cues it holds
pub(super) fn target(workspace: &Workspace, groups: &Groups, cue: &Cue) -> String {
    match cue.kind {
        Kind::Audio | Kind::Video | Kind::Picture => cue
            .file_name()
            .and_then(|name| Path::new(name).file_name())
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
        Kind::Fade | Kind::Start | Kind::Stop | Kind::Pause | Kind::Devamp | Kind::GoTo => {
            workspace.target(cue).map_or_else(String::new, |target| {
                format!("{}{}", number(target), elsewhere(workspace, cue, target))
            })
        }
        // Counts every instrument that the cue names, patched or not, as the
        // Levels tab lists them
        Kind::Light => count(cue.lights.len(), "instrument"),
        Kind::Network => one_line(&cue.message),
        Kind::Group => count(groups.members(cue.id).len(), "cue"),
        Kind::Wait | Kind::Memo => String::new(),
    }
}

/// Tells ` in Effects` for a `target` that lies in a cue list other than
/// that of `cue`, where its number could be mistaken for one of this list,
/// and nothing for one in the same list
fn elsewhere(workspace: &Workspace, cue: &Cue, target: &Cue) -> String {
    let own = workspace.list_of(cue.id);
    workspace
        .list_of(target.id)
        .filter(|other| own.is_some_and(|own| own != *other))
        .map(|other| format!(" in {}", one_line(&workspace.lists()[other].name)))
        .unwrap_or_default()
}

/// Formats a level in dB without the unit, as the level field holds it.
/// Silence reads `-inf`, which every level field also takes as typed.
pub(super) fn decibels(db: f32) -> String {
    if db <= SILENCE {
        "-inf".to_owned()
    } else {
        format!("{db:+.1}")
    }
}

/// Counts things by name, as in `1 cue` and `2 cues`
pub(super) fn count(number: usize, noun: &str) -> String {
    let plural = if number == 1 { "" } else { "s" };
    format!("{number} {noun}{plural}")
}

pub(super) fn phase(active: &Active) -> String {
    match active.phase() {
        Phase::Scheduled { starts_in: wait } | Phase::PreWait { remaining: wait } => {
            format!("in {}", clock(wait))
        }
        Phase::Running { remaining } => format!("{} left", clock(remaining)),
        Phase::Looping => "looping".to_owned(),
        Phase::Showing => "showing".to_owned(),
        Phase::Endless => "running".to_owned(),
    }
}

/// Describes an active cue in full for the list of running cues, with the
/// time left, a pause, and the level first, in the order that the list drops
/// them from the end when they do not fit, so that a fade loses its word
/// before the level that shows it
pub(super) fn activity(active: &Active, cue: &Cue) -> String {
    let paused = if active.paused { ", paused" } else { "" };
    let fading = active.fade.map_or("", |_| ", fading");
    let level = if cue.sounds() {
        format!(", {} dB", decibels(active.level.max(SILENCE)))
    } else {
        String::new()
    };
    format!("{}{paused}{level}{fading}", phase(active))
}

pub(super) fn number(cue: &Cue) -> String {
    if cue.number.trim().is_empty() {
        format!("\"{}\"", one_line(&cue.name))
    } else {
        cue.number.clone()
    }
}
