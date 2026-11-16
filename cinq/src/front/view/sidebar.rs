// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue lists, running cues, warnings, and the audio device, all visible at
//! the same time, with the warnings above the audio device, which a show sets
//! once, so that the first warning shows whole at the least height of the
//! window

mod devices;

use iced::{
    Element, Fill,
    widget::{column, progress_bar, radio, row, rule, scrollable, text},
};

use super::{
    SIDEBAR_SCROLL, SPACING, button,
    faults::{self, Warning},
    format, outputs, warnings,
};
use crate::{
    back::{Active, Cue},
    front::{App, Message},
};

/// Thickness of the progress bars of running cues, thin enough that several
/// running cues fit in the sidebar at once
const BAR: f32 = 10.0;

/// Most warnings that the sidebar lists, so that the Default device list
/// below them stays in view, the rest being counted in a line of their own
const WARNINGS: usize = 3;

pub(super) fn view(app: &App, width: f32) -> Element<'_, Message> {
    let workspace = &app.workspace;
    let lists = workspace.lists().iter().enumerate().map(|(index, list)| {
        radio(
            format!("{} ({})", list.name, list.cues.len()),
            index,
            (!app.outputs_shown).then_some(app.list),
            Message::SwitchList,
        )
        .into()
    });
    let active = workspace
        .active()
        .iter()
        .filter_map(|active| workspace.cue(active.id).map(|cue| active_cue(active, cue)));
    let device = faults::warnings(app).into_iter().map(|warning| {
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
                format!("{}: {state} ({count}){reason}", outputs::device(device))
            }
            Warning::Channels { output, fault } => {
                format!("{}: {}", output.name, fault.state().to_lowercase())
            }
        };
        button(text(told))
            .on_press(Message::InspectOutput(warning.output()))
            .width(Fill)
            .into()
    });
    let files = [&app.file_problem, &app.keep_problem, &app.device_problem]
        .into_iter()
        .flatten()
        .map(|problem| text(problem).style(text::warning).into());
    let settings = app
        .config
        .warnings()
        .map(|warning| text(warning).style(text::warning).into());
    let count = warnings(app);
    let warned = text(format!("Warnings ({count})")).style(if count > 0 {
        text::warning
    } else {
        text::default
    });
    let problems = workspace.problems().map(|(cue, problem)| {
        button(text(format!("{cue}: {problem}")))
            .on_press(Message::Select(cue.id))
            .width(Fill)
            .into()
    });
    let stranded = workspace.stranded().map(|cue| {
        button(text(format!("{cue}: sends to no output")))
            .on_press(Message::Select(cue.id))
            .width(Fill)
            .into()
    });
    let everything: Vec<Element<'_, Message>> = device
        .chain(files)
        .chain(settings)
        .chain(problems)
        .chain(stranded)
        .collect();
    let (shown, left) = limited(everything);
    let more = (left > 0).then(|| text(format!("and {left} more")).style(text::warning).into());
    scrollable(
        column![
            text("Cue lists"),
            column(lists).spacing(SPACING),
            rule::horizontal(1),
            radio(
                format!("Outputs ({})", workspace.outputs().len()),
                (),
                app.outputs_shown.then_some(()),
                |()| Message::ShowOutputs,
            ),
            rule::horizontal(1),
            text(format!("Active ({})", workspace.active().len())),
            column(active).spacing(SPACING),
            rule::horizontal(1),
            warned,
            column(shown.into_iter().chain(more)).spacing(SPACING),
            rule::horizontal(1),
            text("Default device"),
            devices::view(app),
        ]
        .spacing(SPACING)
        .padding(SPACING)
        .width(Fill),
    )
    .spacing(0)
    .id(SIDEBAR_SCROLL)
    .width(width)
    .height(Fill)
    .into()
}

fn active_cue<'a>(active: &Active, cue: &Cue) -> Element<'a, Message> {
    column![
        row![
            text(cue.to_string()).width(Fill),
            button("Stop").on_press(Message::Stop(cue.id)),
        ]
        .spacing(SPACING),
        progress_bar(0.0..=1.0, active.progress()).girth(BAR),
        text(format::activity(active, cue)).style(if active.paused {
            text::warning
        } else {
            text::success
        }),
    ]
    .spacing(SPACING / 2.0)
    .into()
}

/// Splits `warnings` into the first that the sidebar lists and how many it
/// leaves out
fn limited<T>(warnings: Vec<T>) -> (Vec<T>, usize) {
    let left = warnings.len().saturating_sub(WARNINGS);
    (warnings.into_iter().take(WARNINGS).collect(), left)
}

#[cfg(test)]
mod tests {
    use super::limited;

    #[test]
    fn the_sidebar_lists_three_warnings_and_counts_the_rest() {
        assert_eq!(limited(vec![1, 2, 3]), (vec![1, 2, 3], 0));
        assert_eq!(limited(vec![1, 2, 3, 4, 5, 6, 7, 8]), (vec![1, 2, 3], 5));
        assert_eq!(limited(Vec::<u8>::new()), (Vec::new(), 0));
    }
}
