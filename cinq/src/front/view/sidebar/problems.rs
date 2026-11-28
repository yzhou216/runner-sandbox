// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Warnings of the sidebar: a header that counts them and the first few,
//! each of which selects what it is about when pressed

use iced::{
    Element, Fill,
    widget::{column, text, text::Wrapping},
};

use super::super::{
    SPACING, button,
    devices::named,
    faults::{self, Warning},
    format, warnings,
};
use crate::{
    back::one_line,
    front::{App, Message},
};

/// Most warnings that the sidebar lists, so that the Default device list
/// below them stays in view, the rest being counted in a line of their own
const WARNINGS: usize = 3;

pub(super) fn header(app: &App) -> Element<'_, Message> {
    let count = warnings(app);
    text(format!("Warnings ({count})"))
        .style(if count > 0 {
            text::warning
        } else {
            text::default
        })
        .into()
}

pub(super) fn list(app: &App) -> Option<Element<'_, Message>> {
    let everything: Vec<Element<'_, Message>> = outputs_failing(app)
        .chain(network(app))
        .chain(stage(app))
        .chain(files(app))
        .chain(ignored(app))
        .chain(settings(app))
        .chain(cues(app))
        .collect();
    let (shown, left) = limited(everything);
    let more = (left > 0).then(|| text(format!("and {left} more")).style(text::warning).into());
    (!shown.is_empty()).then(|| {
        column(shown.into_iter().chain(more))
            .spacing(SPACING)
            .into()
    })
}

fn outputs_failing(app: &App) -> impl Iterator<Item = Element<'_, Message>> {
    faults::warnings(app).into_iter().map(|warning| {
        let told = match &warning {
            Warning::Device {
                device,
                fault,
                outputs: affected,
            } => {
                let state = fault.state().to_lowercase();
                let reason = fault
                    .reason()
                    .map(|reason| format!(": {reason}"))
                    .unwrap_or_default();
                let count = format::count(affected.len(), "output");
                format!("{}: {state} ({count}){reason}", named(app, device))
            }
            Warning::Channels { output, fault } => {
                format!(
                    "{}: {}",
                    one_line(&output.name),
                    fault.state().to_lowercase()
                )
            }
        };
        pressing(told, Message::InspectOutput(warning.output()))
    })
}

fn network(app: &App) -> impl Iterator<Item = Element<'_, Message>> {
    app.problems
        .network
        .iter()
        .filter_map(|(id, problem)| {
            app.workspace
                .cue(*id)
                .map(|cue| pressing(format!("{cue}: {problem}"), Message::Select(cue.id)))
        })
        .chain(app.problems.light.iter().map(|problem| {
            text(format!("Light output: {problem}"))
                .style(text::danger)
                .into()
        }))
        .chain(
            app.problems
                .osc
                .iter()
                .map(|problem| text(problem).style(text::danger).into()),
        )
}

fn stage(app: &App) -> impl Iterator<Item = Element<'_, Message>> {
    app.stage
        .missing(&app.workspace)
        .then(|| text("Stage window closed").style(text::warning).into())
        .into_iter()
}

fn files(app: &App) -> impl Iterator<Item = Element<'_, Message>> {
    [&app.problems.file, &app.problems.keep, &app.problems.device]
        .into_iter()
        .flatten()
        .map(|problem| text(problem).style(text::warning).into())
}

/// Returns the latest ignored OSC command alone, whatever the flood
fn ignored(app: &App) -> impl Iterator<Item = Element<'_, Message>> {
    app.problems
        .osc_ignored
        .iter()
        .map(|line| text(line).style(text::warning).into())
}

fn settings(app: &App) -> impl Iterator<Item = Element<'_, Message>> {
    app.config
        .warnings()
        .map(|warning| text(warning).style(text::warning).into())
}

fn cues(app: &App) -> impl Iterator<Item = Element<'_, Message>> {
    let workspace = &app.workspace;
    let problems = workspace
        .problems()
        .map(|(cue, problem)| pressing(format!("{cue}: {problem}"), Message::Select(cue.id)));
    let stranded = workspace.stranded().map(|cue| {
        pressing(
            format!("{cue}: sends to no output"),
            Message::Select(cue.id),
        )
    });
    let unlit = workspace
        .unlit_cues()
        .map(|(cue, unlit)| pressing(format!("{cue}: {unlit}"), Message::Select(cue.id)));
    problems.chain(stranded).chain(unlit)
}

/// Returns a warning that `message` answers when it is pressed.  A word that
/// no space breaks, as the ID of a device is, breaks within itself rather than
/// being cut.
fn pressing(told: String, message: Message) -> Element<'static, Message> {
    button(text(told).wrapping(Wrapping::WordOrGlyph))
        .on_press(message)
        .width(Fill)
        .into()
}

/// Splits `warnings` into the first that the sidebar lists and how many it
/// leaves out, which is never exactly one, since the line that counts it
/// takes as much room as the warning does
pub(super) fn limited<T>(warnings: Vec<T>) -> (Vec<T>, usize) {
    let shown = if warnings.len() == WARNINGS + 1 {
        warnings.len()
    } else {
        WARNINGS
    };
    let left = warnings.len().saturating_sub(shown);
    (warnings.into_iter().take(shown).collect(), left)
}
