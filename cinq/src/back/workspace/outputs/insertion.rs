// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Adding and pasting outputs, which give each a name that no output has

use super::{OutputChange, Workspace};
use crate::back::{Output, OutputId};

impl Workspace {
    /// Returns the first name that `name` makes from the numbers from one on
    /// and that neither an output of the show nor one of `also` has
    fn unused_name(&self, also: &[Output], name: impl Fn(usize) -> String) -> String {
        (1..)
            .map(name)
            .find(|name| {
                self.outputs
                    .iter()
                    .chain(also)
                    .all(|output| output.name != *name)
            })
            .expect("the outputs run out before the names do")
    }

    /// Adds `count` outputs in a row at `at` among the outputs, or at the
    /// end past them, as one change, each on the default device and its first
    /// two channels, under the first name of the form Output 1 that no output
    /// has
    pub(in crate::back::workspace) fn added_output(
        self,
        at: usize,
        count: usize,
        anchor: Option<OutputId>,
    ) -> Self {
        let added = (0..count).fold(Vec::<Output>::new(), |added, offset| {
            let id = OutputId(self.next_output + offset as u32);
            let name = self.unused_name(&added, |number| format!("Output {number}"));
            let output = Output {
                name,
                new_cues: None,
                ..Output::main(id)
            };
            added.into_iter().chain([output]).collect()
        });
        self.inserted(at, added, false, anchor)
    }

    /// Pastes `outputs` in a row at `at` among the outputs, as one change.
    /// With `restore`, an output whose identity no output of the show has
    /// takes it back, with the level of its new cues, so that the cues that
    /// sent to it send to it again, as a line cut and pasted back in Vim is
    /// the line that was.  Every other output is a copy: it has an identity of
    /// its own, no cue sends to it, and no new cue sends to it either.  A name
    /// that an output has gets the lowest number from 2 on that makes it free,
    /// after a number that the name ends in after a space gives way to the new
    /// one, so that a copy of House R 2 is House R 3.
    pub(in crate::back::workspace) fn pasted_outputs(
        self,
        at: usize,
        outputs: Vec<Output>,
        restore: bool,
        anchor: Option<OutputId>,
    ) -> Self {
        let (pasted, _) = outputs.into_iter().fold(
            (Vec::<Output>::new(), self.next_output),
            |(pasted, next), output| {
                let back = restore
                    && self.output(output.id).is_none()
                    && pasted.iter().all(|placed| placed.id != output.id);
                let base = numbered_from(&output.name);
                let name = self.unused_name(&pasted, |number| match number {
                    1 => output.name.clone(),
                    number => format!("{base} {number}"),
                });
                let placed = if back {
                    Output { name, ..output }
                } else {
                    Output {
                        id: OutputId(next),
                        name,
                        new_cues: None,
                        ..output
                    }
                };
                let next = if back { next } else { next + 1 };
                (pasted.into_iter().chain([placed]).collect(), next)
            },
        );
        self.inserted(at, pasted, true, anchor)
    }

    /// Puts `added` among the outputs at `at`, or at the end past them, as
    /// one change that tells whether they were `pasted`, and nothing when
    /// there is nothing to put
    fn inserted(
        self,
        at: usize,
        added: Vec<Output>,
        pasted: bool,
        anchor: Option<OutputId>,
    ) -> Self {
        if added.is_empty() {
            return self;
        }
        let at = at.min(self.outputs.len());
        let outputs = [&self.outputs[..at], &added, &self.outputs[at..]].concat();
        OutputChange::to(outputs)
            .with_pasted(pasted)
            .with_anchor(anchor)
            .recorded_on(self)
    }
}

/// Returns `name` without the space and the number that end it, which a
/// numbered copy replaces with its own, unless nothing but the number is
/// left
fn numbered_from(name: &str) -> &str {
    name.rsplit_once(' ')
        .filter(|(base, number)| {
            !base.trim().is_empty()
                && !number.is_empty()
                && number.chars().all(|digit| digit.is_ascii_digit())
        })
        .map_or(name, |(base, _)| base)
}
