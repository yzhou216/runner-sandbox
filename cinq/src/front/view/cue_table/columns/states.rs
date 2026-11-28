// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Lines of the State column, with the colors that tell their urgency, and
//! what a collapsed group tells of the cues that it hides

mod look;

use iced::{Theme, widget::text};
use itertools::Itertools;

pub(in crate::front::view::cue_table) use look::Look;

use super::Column;
use crate::{
    back::{Cue, Groups, Kind, Problem, Unlit, Workspace},
    front::view::{format, theme},
};

/// Line of the State column with the style of its color
type Line = (String, fn(&Theme) -> text::Style);

/// Returns the lines of the State column for a cue as its `look` describes
/// the row.  NEXT comes first, so that the mark stays where GO lands whatever
/// else the cue needs to say.  A cue whose outputs all are failing plays on,
/// in silence, which a last line of its own tells in yellow, so that it hides
/// neither the state of a cue that runs nor its time left.  A network cue
/// whose destination is unreached tells so in the same way, until a message
/// gets there.
pub(in crate::front::view::cue_table) fn lines(
    workspace: &Workspace,
    cue: &Cue,
    look: Look,
) -> Vec<Line> {
    let mark = look
        .next
        .then(|| ("NEXT".to_owned(), text::primary as fn(&Theme) -> _));
    let state = state(workspace, cue, look.hidden, look.selected);
    let sendable = workspace.problem(cue).is_none();
    let warning = |told: &str| (told.to_owned(), text::warning as fn(&Theme) -> _);
    let failed = (sendable && workspace.sends_to_failing(cue, look.failing))
        .then(|| warning("OUTPUT FAILED"));
    let unsent = (sendable && workspace.sends_to_failed(cue, look.unreached))
        .then(|| warning("SEND FAILED"));
    mark.into_iter()
        .chain((!state.0.is_empty()).then_some(state))
        .chain(failed)
        .chain(unsent)
        .collect()
}

/// Describes a cue in the State column, in the color of what it needs.  A
/// file that is still loading stops the cue from playing, but it needs
/// nothing but time.  The list of running cues tells more, such as where a
/// paused cue stopped.  A cue that a fade or Panic is fading reads fading,
/// since the time left of its phase says nothing of that.  A cue that sends
/// only to outputs that are gone, or a light cue that sets no instrument or
/// names one that the patch lacks, reads so in yellow, and GO does not skip
/// it.  Red turns
/// lighter where the cue is `selected`, so that it stays legible on the
/// selected row.  A collapsed group reads the `hidden` problem of a cue that
/// it hides as that cue would, so that GO does not fire a broken cue out of
/// sight, but gives way to the state of the group while it runs.
pub(in crate::front::view::cue_table) fn state(
    workspace: &Workspace,
    cue: &Cue,
    hidden: Option<Problem>,
    selected: bool,
) -> Line {
    let problem = |problem: Problem| -> Line {
        let words = match problem {
            Problem::NotLoaded => return ("loading".to_owned(), text::default),
            Problem::NoFile => "NO FILE",
            Problem::NoTarget => "NO TARGET",
            Problem::WrongTarget | Problem::NotAudioOrVideo => "BAD TARGET",
            Problem::InTarget => "IN TARGET",
            Problem::NotLooping => "NO LOOP",
            Problem::NoDestination => "NO DESTINATION",
            Problem::BadPort => "BAD PORT",
            Problem::NoMessage | Problem::OpenQuote => "NO MESSAGE",
            Problem::LoadFailed(_)
            | Problem::EmptyClip
            | Problem::PastFile
            | Problem::EmptyLoop
            | Problem::ShortLoop => "BROKEN",
        };
        (words.to_owned(), theme::danger(selected))
    };
    match (workspace.problem(cue), workspace.active_of(cue.id)) {
        (Some(own), _) => problem(own),
        (None, Some(active)) if active.paused => ("paused".to_owned(), text::warning),
        // Fade leaves no room for the time left, which the fade cue tells
        (None, Some(active)) if active.fade.is_some() => ("fading".to_owned(), text::success),
        (None, Some(active)) => (format::phase(active), text::success),
        (None, None) => match hidden {
            Some(hidden) => problem(hidden),
            None if workspace.sends_nowhere(cue) => ("NO OUTPUT".to_owned(), text::warning),
            None => match workspace.unlit(cue) {
                Some(Unlit::Empty) => ("NO LEVELS".to_owned(), text::warning),
                Some(Unlit::Unpatched(_)) => ("UNPATCHED".to_owned(), text::warning),
                None => (String::new(), text::default),
            },
        },
    }
}

/// Returns the lines that the cue list shows for `cue`: [`lines`] and, below
/// them, the runs and warnings that a collapsed group hides.  The `look`
/// needs no hidden problem, which the function finds.
pub(in crate::front::view::cue_table) fn shown_lines(
    workspace: &Workspace,
    groups: &Groups,
    cue: &Cue,
    look: Look,
) -> Vec<Line> {
    let look = look.with_hidden(hidden_problem(workspace, groups, cue));
    lines(workspace, cue, look)
        .into_iter()
        .chain(hidden_runs(workspace, groups, cue))
        .chain(hidden_warnings(workspace, groups, cue, look))
        .collect()
}

/// Returns the yellow warnings that the cues of the collapsed group `cue`
/// would tell, each once, so that folding the group hides neither a silent
/// cue nor a light that sets nothing.  A cue with a problem is told by the
/// problem instead.
fn hidden_warnings(workspace: &Workspace, groups: &Groups, cue: &Cue, look: Look) -> Vec<Line> {
    const WARNINGS: [&str; 5] = [
        "OUTPUT FAILED",
        "SEND FAILED",
        "NO OUTPUT",
        "NO LEVELS",
        "UNPATCHED",
    ];
    let member_look = Look::default()
        .with_failing(look.failing)
        .with_unreached(look.unreached);
    hidden(groups, cue)
        .iter()
        .filter(|member| member.kind != Kind::Group && workspace.problem(member).is_none())
        .flat_map(|member| lines(workspace, member, member_look))
        .filter(|(told, _)| WARNINGS.contains(&told.as_str()))
        .unique_by(|(told, _)| told.clone())
        .collect()
}

/// Returns the lines that count the cues of the collapsed group `cue` that
/// run, the paused ones first in yellow, so that folding the group hides no
/// sound or picture that plays.  A group that runs itself tells its own state
/// instead.
fn hidden_runs(workspace: &Workspace, groups: &Groups, cue: &Cue) -> Vec<Line> {
    let members = if workspace.active_of(cue.id).is_none() {
        hidden(groups, cue)
    } else {
        &[]
    };
    let runs = members
        .iter()
        .filter(|member| member.kind != Kind::Group)
        .filter_map(|member| workspace.active_of(member.id))
        .map(|active| active.paused)
        .collect::<Vec<_>>();
    let paused = runs.iter().filter(|&&paused| paused).count();
    [
        (paused, "paused", text::warning as fn(&Theme) -> _),
        (runs.len() - paused, "running", text::success),
    ]
    .into_iter()
    .filter(|&(count, ..)| count > 0)
    .map(|(count, word, style)| (format!("{count} {word}"), style))
    .collect()
}

/// Returns the problem of the first cue that the collapsed group `cue` hides
/// which has one
pub(in crate::front::view::cue_table) fn hidden_problem<'a>(
    workspace: &Workspace,
    groups: &Groups<'a>,
    cue: &Cue,
) -> Option<Problem<'a>> {
    hidden(groups, cue)
        .iter()
        .find_map(|member| workspace.problem(member))
}

/// Returns the cues that the collapsed group `cue` hides
fn hidden<'a>(groups: &Groups<'a>, cue: &Cue) -> &'a [Cue] {
    if cue.kind == Kind::Group && cue.collapsed {
        groups.members(cue.id)
    } else {
        &[]
    }
}

/// Returns how many cues the collapsed group `cue` hides, as ` (10 cues)`
/// for the end of its name, which a list with no room for the Target column,
/// where the count belongs, shows
pub(in crate::front::view::cue_table) fn counted(
    workspace: &Workspace,
    groups: &Groups,
    cue: &Cue,
) -> Option<String> {
    (cue.kind == Kind::Group && cue.collapsed)
        .then(|| format!(" ({})", Column::Target.text(workspace, groups, cue)))
}
