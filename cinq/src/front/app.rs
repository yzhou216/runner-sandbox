// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! State that the interface keeps of its own, apart from the show, and what
//! it reads of the selection and the cue list

use std::time::{Duration, Instant};

use iced::window;

use super::{
    config::Config,
    keymap::Press,
    message::{Setting, Tab},
    update::{Clipboard, Hints, History, Measures, Reader, Search, Searched, Show, Stage},
};
use crate::back::{CueId, CueList, Kind, OutputId, Workspace};

mod keyboard;
mod notices;
mod problems;

pub(super) use keyboard::{Held, Keyboard};
pub(super) use notices::Notices;
pub(super) use problems::Problems;

#[derive(Default)]
pub(super) struct App {
    pub(super) workspace: Workspace,
    /// Main window, once it has been asked for, whose closing quits cinq
    pub(super) main: Option<window::Id>,
    pub(super) stage: Stage,
    /// Windows that are minimized, which iced does not draw
    pub(super) minimized: Vec<window::Id>,
    pub(super) show: Show,
    /// What has gone wrong, which the sidebar warns of
    pub(super) problems: Problems,
    /// What the latest action told, which the status bar shows
    pub(super) notices: Notices,
    /// Way to read the unsaved changes kept for shows, if the platform has
    /// a directory to keep them in
    pub(super) reader: Option<Reader>,
    /// Interval at which to look for news of the network and light output,
    /// while either has any
    pub(super) network_checks: Option<Duration>,
    /// ID and name of each audio device, as they were listed last
    pub(super) audio_devices: Vec<(String, String)>,
    /// Number of the latest asking for the audio devices, whose answer alone
    /// applies, since the answers come in any order
    pub(super) device_serial: u64,
    /// Rows that a half page moves in the cue list, once a count has set them
    pub(super) scroll: Option<usize>,
    /// What the latest measures of the cue list and the help page found
    pub(super) measures: Measures,
    pub(super) list: usize,
    /// Places that jumps have left, for going back to them
    pub(super) history: History,
    /// Cue shown in the inspector
    pub(super) selected: Option<CueId>,
    /// Text of a setting as the operator types it, which its field shows in
    /// place of the setting until the operator does anything else
    pub(super) typed: Option<(Setting, String)>,
    /// Whether the outputs of the show show in place of the cue list
    pub(super) outputs_shown: bool,
    /// Output that the inspector shows while the outputs show
    pub(super) output: Option<OutputId>,
    pub(super) tab: Tab,
    pub(super) clipboard: Clipboard,
    /// Kind of cue that Add cue creates
    pub(super) new_kind: Kind,
    /// Time of the last clock tick, or `None` while the clock is stopped
    pub(super) last_tick: Option<Instant>,
    pub(super) config: Config,
    /// Keys of a command typed so far, such as a count or the `g` of `g g`
    pub(super) keys: Vec<Press>,
    pub(super) search: Option<Search>,
    /// Text of the search accepted last, which the keys of the next and
    /// previous match look for
    pub(super) searched: Searched,
    /// Labels of hint mode, while it is on
    pub(super) hints: Option<Hints>,
    pub(super) help: bool,
    /// What has the keyboard, which the status bar hints the keys of
    pub(super) keyboard: Keyboard,
    /// Whether the files of audio, video, and picture cues are being loaded
    pub(super) loading: bool,
    /// Whether the files that could not be played are being read again
    pub(super) rechecking: bool,
}

impl App {
    pub(super) fn current(&self) -> &CueList {
        &self.workspace.lists()[self.list]
    }

    pub(super) fn selected_here(&self) -> Option<CueId> {
        self.selected.filter(|_| self.selected_index().is_some())
    }

    /// Returns the place of the selected cue among all the cues of the list
    pub(super) fn selected_index(&self) -> Option<usize> {
        self.selected.and_then(|id| self.current().position(id))
    }

    /// Returns the row that shows the selected cue, counted from zero among
    /// the rows that the table shows, which is the row of the collapsed group
    /// that holds the cue where one does
    pub(super) fn selected_row(&self) -> Option<usize> {
        let list = self.current();
        let groups = list.groups();
        let standing = *groups.shown_as().get(self.selected_index()?)?;
        groups.shown().binary_search(&standing).ok()
    }

    pub(super) fn editable(&self) -> bool {
        !self.workspace.show_mode()
    }

    pub(super) fn running(&self) -> bool {
        self.workspace.active().iter().any(|active| !active.paused)
    }

    pub(super) fn moving_pictures(&self) -> bool {
        self.workspace
            .active()
            .iter()
            .any(|active| active.shows_video() && !active.paused)
    }
}
