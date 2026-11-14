// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Warnings about the outputs that cannot play, which the sidebar lists and
//! the status bar counts
//!
//! The outputs on a device that cinq tries again share one warning, since
//! nine outputs of a missing Dante Virtual Soundcard tell one thing, and a
//! warning each would push every other warning out of sight.  An output
//! past the channels of its device gets a warning of its own, since only
//! its channels need changing.

#[cfg(test)]
mod tests;

use crate::{
    back::{Device, Fault, Output, OutputId},
    front::App,
};

/// Warning about outputs that cannot play
#[derive(Debug, PartialEq)]
pub(super) enum Warning<'a> {
    /// Outputs on a device that cinq tries again, which share its fault
    Device {
        device: &'a Device,
        fault: &'a Fault,
        outputs: Vec<OutputId>,
    },
    /// Output past the channels of its device
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

/// Returns the warnings about the outputs of `app` that cannot play, in the
/// order of the outputs
pub(super) fn warnings(app: &App) -> Vec<Warning<'_>> {
    let faulty = app.workspace.outputs().iter().filter_map(|output| {
        let (_, fault) = app.audio_problems.iter().find(|(id, _)| *id == output.id)?;
        Some((output, fault))
    });
    faulty.fold(Vec::new(), |warnings, (output, fault)| {
        if !fault.retried() {
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
    })
}

/// Returns the fault of the output of `id`, if it cannot play
pub(super) fn fault(app: &App, id: OutputId) -> Option<&Fault> {
    app.audio_problems
        .iter()
        .find(|(output, _)| *output == id)
        .map(|(_, fault)| fault)
}
