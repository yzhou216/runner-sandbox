// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Copying, cutting, and pasting outputs
//!
//! Outputs have a register of their own beside that of cues, so that copying
//! an output keeps the cues that were copied, as Vim keeps its named
//! registers apart.  A cut output that is pasted back into the show it came
//! from takes back its identity, so that the cues that sent to it send to it
//! again.  A copy gets an identity of its own.

use super::editing::LIMIT;
use crate::{
    back::{Command, Output},
    front::{
        App,
        update::{
            Clipped,
            editing::{Clipboard, EMPTY, Register},
        },
        view::format,
    },
};

impl App {
    /// Copies the selected output and the outputs below it, as many as
    /// `count` holds, in place of what the register held.  Copying changes no
    /// show, so that show mode leaves it.
    pub(in crate::front::update) fn copied_outputs(self, count: usize) -> Self {
        let outputs = self
            .output_row()
            .map(|first| {
                self.workspace.outputs()[first..]
                    .iter()
                    .take(count)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        self.registered(outputs, "copied", false, None)
    }

    /// Cuts as [`App::copied_outputs`] copies, and deletes the outputs
    /// copied, apart from a count that reaches all of them, which cuts none
    pub(in crate::front::update) fn cut_outputs(self, count: usize) -> Self {
        let before = self.workspace.outputs().to_vec();
        let (app, taken) = self.taking_outputs(count, Command::CutOutputs);
        let told = format::consequences(&app.workspace, &before, app.workspace.outputs());
        app.registered(taken, "cut", true, told)
    }

    /// Puts `outputs` in the register, if there are any, and tells what went
    /// there by the `verb`, in lower case, after `facts` of what it did to the
    /// cues
    fn registered(
        self,
        outputs: Vec<Output>,
        verb: &str,
        cut: bool,
        facts: Option<String>,
    ) -> Self {
        if outputs.is_empty() {
            return self;
        }
        let clipped = Clipped {
            told: format::clipped_outputs(verb, &outputs, facts),
            cut,
        };
        Self {
            clipboard: Clipboard {
                outputs: Register {
                    show: self.show.id(),
                    cut,
                    outputs,
                },
                ..self.clipboard
            },
            notices: self.notices.with_clipped(Some(clipped)),
            ..self
        }
    }

    /// Pastes the register `count` times in a row below the selected output,
    /// or above it, and selects the first output pasted, as far as
    /// [`LIMIT`] allows.  An empty register pastes nothing and tells so.
    pub(super) fn with_pasted_outputs(self, above: bool, count: usize) -> Self {
        if !self.editable() {
            return self;
        }
        let register = &self.clipboard.outputs;
        let outputs: Vec<Output> = (0..count.min(LIMIT))
            .flat_map(|_| register.outputs.iter().cloned())
            .take(LIMIT)
            .collect();
        if outputs.is_empty() {
            return Self {
                notices: self.notices.with_remark(Some(EMPTY)),
                ..self
            };
        }
        let restore = register.cut && register.show == self.show.id();
        let at = self.insertion(above);
        let anchor = self.output;
        let app = self.send(Command::PasteOutputs {
            at,
            outputs,
            restore,
            anchor,
        });
        let row = at.min(app.workspace.outputs().len().saturating_sub(1));
        app.selecting(Some(row))
    }
}
