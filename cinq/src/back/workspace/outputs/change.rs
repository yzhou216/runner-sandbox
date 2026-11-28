// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The change of the outputs to a new set, with what the history records of it

use std::mem;

use super::super::{Change, Workspace};
use crate::back::{Output, OutputEdit, OutputId};

/// The outputs that the show takes, with what the history records of how
/// it came by them: by default a change that is no edit of one setting,
/// no cut, no paste, and has no output to keep the selection on
pub(super) struct OutputChange {
    outputs: Vec<Output>,
    setting: Option<(OutputId, mem::Discriminant<OutputEdit>)>,
    cut: bool,
    pasted: bool,
    anchor: Option<OutputId>,
}

impl OutputChange {
    pub(super) fn to(outputs: Vec<Output>) -> Self {
        Self {
            outputs,
            setting: None,
            cut: false,
            pasted: false,
            anchor: None,
        }
    }

    /// Tells that the change edits one setting of an output, so that
    /// edits of the same setting join into one change
    pub(super) fn with_setting(self, id: OutputId, edit: mem::Discriminant<OutputEdit>) -> Self {
        Self {
            setting: Some((id, edit)),
            ..self
        }
    }

    pub(super) fn with_cut(self, cut: bool) -> Self {
        Self { cut, ..self }
    }

    pub(super) fn with_pasted(self, pasted: bool) -> Self {
        Self { pasted, ..self }
    }

    pub(super) fn with_anchor(self, anchor: Option<OutputId>) -> Self {
        Self { anchor, ..self }
    }

    /// Records the change of the outputs of `workspace`, unless they stay
    /// as they are, which leaves redo alone, as a cue edit that changes
    /// nothing does
    pub(super) fn recorded_on(self, workspace: Workspace) -> Workspace {
        if self.outputs == workspace.outputs {
            return workspace;
        }
        let change = Change::Outputs {
            before: workspace.outputs.clone(),
            after: self.outputs.clone(),
            setting: self.setting,
            cut: self.cut,
            pasted: self.pasted,
            anchor: self.anchor,
        };
        workspace.with_outputs(self.outputs).recorded(change)
    }
}
