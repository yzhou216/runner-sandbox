// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What has gone wrong, which the sidebar warns of

use std::collections::BTreeSet;

use crate::back::{CueId, Fault, OutputId};

#[derive(Default)]
pub(in crate::front) struct Problems {
    /// What went wrong with show files last, until opening or saving one
    /// succeeds
    pub(in crate::front) file: Option<String>,
    /// Whether `file` is a show that failed to open, which leaves the show on
    /// screen, an empty one when cinq has just started
    pub(in crate::front) open_failed: bool,
    /// Why the keeper failed to keep unsaved changes last, until opening or
    /// saving a show succeeds.  It arrives late and about an earlier change,
    /// so it has a slot of its own.
    pub(in crate::front) keep: Option<String>,
    /// Why each output that cannot play cannot, by its identity
    pub(in crate::front) audio: Vec<(OutputId, Fault)>,
    /// Why messages of network cues failed, until one gets there, each with
    /// the first cue that names the destination
    pub(in crate::front) network: Vec<(CueId, String)>,
    /// Destinations that messages of network cues failed to reach, which
    /// each cue that names one tells on its row
    pub(in crate::front) network_failed: BTreeSet<String>,
    /// Why no light gets out, until a round of sending gets some out, while
    /// the patch has an instrument
    pub(in crate::front) light: Option<String>,
    /// Why the address of `osc_listen` cannot be listened on, until the
    /// setting changes
    pub(in crate::front) osc: Option<String>,
    /// Latest command from the network that did nothing, as one line, until
    /// a later command acts or the setting changes
    pub(in crate::front) osc_ignored: Option<String>,
    /// Why the configuration file did not keep the audio device chosen
    /// last, until it keeps another
    pub(in crate::front) device: Option<String>,
    /// Why the link that the help page opened last failed to open, which
    /// the page shows while it stays open
    pub(in crate::front) link: Option<String>,
}

impl Problems {
    /// Counts the warnings of the sidebar that these problems make, apart
    /// from those of the outputs that cannot play, which depend on the show
    pub(in crate::front) fn listed(&self) -> usize {
        self.network.len()
            + [
                &self.light,
                &self.osc,
                &self.osc_ignored,
                &self.file,
                &self.keep,
                &self.device,
            ]
            .into_iter()
            .flatten()
            .count()
    }

    pub(in crate::front) fn with_osc(self, osc: Option<String>) -> Self {
        Self { osc, ..self }
    }

    pub(in crate::front) fn with_osc_ignored(self, line: Option<String>) -> Self {
        Self {
            osc_ignored: line,
            ..self
        }
    }

    pub(in crate::front) fn with_link(self, link: Option<String>) -> Self {
        Self { link, ..self }
    }

    /// Returns the problems with the failure to open a link only while `shown`,
    /// as the help page shows it
    pub(in crate::front) fn with_link_while(self, shown: bool) -> Self {
        Self {
            link: self.link.filter(|_| shown),
            ..self
        }
    }

    pub(in crate::front) fn with_device(self, device: Option<String>) -> Self {
        Self { device, ..self }
    }

    /// Returns the problems without those of show files, as opening or saving
    /// a show leaves them
    pub(in crate::front) fn without_files(self) -> Self {
        Self {
            file: None,
            open_failed: false,
            keep: None,
            ..self
        }
    }

    /// Returns the problems with `failed` as why the keeper failed to keep
    /// changes, if it did, which replaces an older reason
    pub(in crate::front) fn with_keep(self, failed: Option<String>) -> Self {
        Self {
            keep: failed.or(self.keep),
            ..self
        }
    }

    /// Returns the problems of the address of `osc_listen` as they stand
    /// after the setting changed or stayed, where `same` tells that it
    /// stayed
    pub(in crate::front) fn with_osc_setting(self, same: bool) -> Self {
        Self {
            osc: self.osc.filter(|_| same),
            osc_ignored: self.osc_ignored.filter(|_| same),
            ..self
        }
    }
}
