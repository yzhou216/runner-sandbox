// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Opening, closing, raising, and fullscreen toggling of the stage window

use iced::{
    Task,
    window::{self, Mode},
};

use super::Stage;
use crate::front::{App, Message};

impl App {
    /// A window that opens fullscreen keeps the keyboard, while the main
    /// window asks for it back from one that opens as a window, which the
    /// window manager may refuse, as those of Wayland do.
    pub(in crate::front::update) fn staged(self) -> (Self, Task<Message>) {
        if let Some(window) = self.stage.window {
            let raising = if self.stage.cued {
                raised(window)
            } else {
                Task::none()
            };
            let stage = Stage {
                cued: false,
                ..self.stage
            };
            return (Self { stage, ..self }, raising);
        }
        let wanted = !self.stage.closed && self.holds_stage_cue();
        if !wanted {
            return (self, Task::none());
        }
        let (window, opened) = window::open(self.stage.settings());
        let main = self.main.filter(|_| !self.stage.cued);
        let opening = opened.then(move |_| main.map_or_else(Task::none, window::gain_focus));
        let stage = Stage {
            window: Some(window),
            cued: false,
            ..self.stage
        };
        (Self { stage, ..self }, opening)
    }

    pub(in crate::front::update) fn closed(self, window: window::Id) -> (Self, Task<Message>) {
        if self.stage.is_window(window) {
            let stage = Stage {
                window: None,
                closed: true,
                keyboard: false,
                ..self.stage
            };
            let app = Self { stage, ..self }.with_minimized(window, false);
            (app, Task::none())
        } else if self.main == Some(window) {
            (self, iced::exit())
        } else {
            (self, Task::none())
        }
    }

    /// Toggles the stage window between fullscreen and a window, or opens it
    /// fullscreen, as a cue that shows on it does, where the operator closed
    /// it, so that the film that runs shows again
    pub(in crate::front::update) fn toggled_fullscreen(self) -> (Self, Task<Message>) {
        match self.stage.window {
            Some(window) => {
                let main = self.main;
                let toggling = window::mode(window).then(move |mode| toggled(window, main, mode));
                (self, toggling)
            }
            None if self.stage.closed && self.holds_stage_cue() => Self {
                stage: self.stage.cued(),
                ..self
            }
            .staged(),
            None => (self, Task::none()),
        }
    }

    fn holds_stage_cue(&self) -> bool {
        self.workspace.cues().any(|cue| cue.kind.on_stage())
    }
}

/// Switches the stage window out of fullscreen, and the main window then asks
/// for the keyboard, which the fullscreen window kept, or switches the stage
/// window into it
pub(super) fn toggled(window: window::Id, main: Option<window::Id>, mode: Mode) -> Task<Message> {
    if mode == Mode::Fullscreen {
        window::set_mode(window, Mode::Windowed)
            .chain(main.map_or_else(Task::none, window::gain_focus))
    } else {
        window::set_mode(window, Mode::Fullscreen)
    }
}

fn raised(window: window::Id) -> Task<Message> {
    window::mode(window)
        .then(move |mode| {
            if mode == Mode::Fullscreen {
                Task::none()
            } else {
                window::set_mode(window, Mode::Fullscreen)
            }
        })
        .chain(window::gain_focus(window))
}
