// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Warnings about the outputs that cannot play, which the sidebar lists and
//! the status bar counts
//!
//! The outputs on a device that fails share one warning, since nine outputs
//! of a missing Dante Virtual Soundcard tell one thing, and a warning each
//! would push every other warning out of sight.  An output past the channels
//! of its device gets a warning of its own, since only its channels need
//! changing.

#[cfg(test)]
mod tests;

use crate::{
    back::{Device, Fault, Output, OutputId},
    front::App,
};

#[derive(Debug, PartialEq)]
pub(super) enum Warning<'a> {
    /// Outputs on a device that fails, which share its fault
    Device {
        device: &'a Device,
        fault: &'a Fault,
        outputs: Vec<OutputId>,
    },
    Channels {
        output: &'a Output,
        fault: &'a Fault,
    },
}

impl Warning<'_> {
    /// Returns the output that the inspector shows for the warning: the
    /// first of its outputs
    pub(super) fn output(&self) -> OutputId {
        match self {
            Warning::Device { outputs, .. } => outputs[0],
            Warning::Channels { output, .. } => output.id,
        }
    }
}

/// Returns the warnings, in the order of the outputs
pub(super) fn warnings(app: &App) -> Vec<Warning<'_>> {
    app.workspace
        .outputs()
        .iter()
        .filter_map(|output| Some((output, fault(app, output.id)?)))
        .fold(Vec::new(), with_output)
}

/// Returns `warnings` with the `output` that fails for `fault` added: a
/// warning of its own for a channel fault, else a place on the warning of its
/// device, which the first output of the device opens
fn with_output<'a>(
    warnings: Vec<Warning<'a>>,
    (output, fault): (&'a Output, &'a Fault),
) -> Vec<Warning<'a>> {
    if matches!(fault, Fault::Channels(_)) {
        return warnings
            .into_iter()
            .chain([Warning::Channels { output, fault }])
            .collect();
    }
    let shared = warnings.iter().any(
        |warning| matches!(warning, Warning::Device { device, .. } if **device == output.device),
    );
    if !shared {
        let warning = Warning::Device {
            device: &output.device,
            fault,
            outputs: vec![output.id],
        };
        return warnings.into_iter().chain([warning]).collect();
    }
    warnings
        .into_iter()
        .map(|warning| match warning {
            Warning::Device {
                device,
                fault,
                outputs,
            } if *device == output.device => Warning::Device {
                device,
                fault,
                outputs: outputs.into_iter().chain([output.id]).collect(),
            },
            warning => warning,
        })
        .collect()
}

/// Returns the outputs that cannot play, which a cue that sends to no other
/// makes no sound on.  An output past the channels of its device is one only
/// where all its channels are past them, since a pair that overlaps the last
/// channel half sounds, which the sidebar warns of already.
pub(super) fn failing(app: &App) -> Vec<OutputId> {
    app.problems
        .audio
        .iter()
        .filter(|(id, fault)| match fault {
            Fault::Channels(count) => app
                .workspace
                .outputs()
                .iter()
                .find(|output| output.id == *id)
                .is_some_and(|output| output.channels.lowest() >= *count),
            _ => true,
        })
        .map(|(id, _)| *id)
        .collect()
}

pub(super) fn fault(app: &App, id: OutputId) -> Option<&Fault> {
    app.problems
        .audio
        .iter()
        .find(|(output, _)| *output == id)
        .map(|(_, fault)| fault)
}
