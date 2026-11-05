// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stage window, which shows the pictures of the video cues that play
//!
//! The stage window opens once the show holds a video cue, and shows black
//! while no video plays.  Closing it closes it until a video cue fires
//! again, which opens it anew, at its first size where the platform puts
//! it, or fullscreen if the operator last made it fullscreen, while closing
//! the main window quits cinq.  Each picture that the video output hands on
//! becomes an image for iced once, as it comes due, and stays until a newer
//! picture of its cue takes its place or the cue stops.  The key of the
//! fullscreen action toggles the window between fullscreen and a window.
//!
//! The interface wakes for each picture as it comes due, and each picture
//! costs a whole update, as each tick of the clock does, which builds the
//! view of the main window again along with that of the stage.  On an
//! Intel Core i7-13700H, the pictures of a video take the thread of the
//! interface about 2 ms each, whether the show holds three cues or 300,
//! while the 40 ticks a second of the clock take 2% of its time with three
//! cues and 10% with 300.

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use iced::{
    Task,
    widget::image,
    window::{self, Mode},
};

use crate::{
    back::{CueId, Kind, Picture, Workspace},
    front::{App, Message, STAGE_SIZE, window_settings},
};

/// Stage window, with the newest picture of each video cue that runs
#[derive(Default)]
pub struct Stage {
    /// Window while it is open
    window: Option<window::Id>,
    /// Whether the operator closed the window, which keeps it closed until
    /// a video cue fires
    closed: bool,
    /// Whether the window has the keyboard, which the window manager may
    /// leave with it once it opens
    keyboard: bool,
    /// Whether the operator last made the window fullscreen, as it opens
    /// again
    fullscreen: bool,
    /// Newest picture of each video cue that runs, as iced draws it
    pictures: HashMap<CueId, image::Handle>,
    /// Seconds of the show clock until the next picture of a video cue
    /// that plays comes due, once that picture has arrived
    due: Option<f64>,
}

impl Stage {
    /// Returns the stage window while it is open
    pub fn window(&self) -> Option<window::Id> {
        self.window
    }

    /// Returns how many seconds of the show clock pass before the next
    /// picture of a video cue that plays comes due, once it has arrived
    pub fn due(&self) -> Option<f64> {
        self.due
    }

    /// Returns the stage with its next picture due in `due` seconds of the
    /// show clock, or with none due
    pub(in crate::front) fn with_due(self, due: Option<f64>) -> Self {
        Self { due, ..self }
    }

    /// Returns whether `window` is the stage window
    pub fn is_window(&self, window: window::Id) -> bool {
        self.window == Some(window)
    }

    /// Returns whether the stage window has the keyboard
    pub fn has_keyboard(&self) -> bool {
        self.keyboard
    }

    /// Returns the stage once `window` has gained the keyboard
    pub(super) fn with_keyboard_in(self, window: window::Id) -> Self {
        Self {
            keyboard: self.is_window(window),
            ..self
        }
    }

    /// Returns the newest picture of the video cue `cue`, if it has one
    pub fn picture(&self, cue: CueId) -> Option<&image::Handle> {
        self.pictures.get(&cue)
    }

    /// Returns the stage with `pictures`, which have come due, in place of
    /// the earlier pictures of their cues, and without the pictures of the
    /// cues that `workspace` no longer runs
    pub(in crate::front) fn with_pictures(
        self,
        workspace: &Workspace,
        pictures: Vec<(CueId, Picture)>,
    ) -> Self {
        let due = pictures.into_iter().map(|(cue, picture)| {
            let image = image::Handle::from_rgba(picture.width, picture.height, picture.pixels);
            (cue, image)
        });
        let pictures = self
            .pictures
            .into_iter()
            .chain(due)
            .filter(|(cue, _)| workspace.active_of(*cue).is_some())
            .collect();
        Self { pictures, ..self }
    }

    /// Returns the stage to open again, as a video cue that fires opens it
    /// after the operator has closed it
    pub(super) fn reopened(self) -> Self {
        Self {
            closed: false,
            ..self
        }
    }

    /// Returns the stage once the operator has made its window fullscreen,
    /// or a window again
    pub(super) fn toggled(self, fullscreen: bool) -> Self {
        Self { fullscreen, ..self }
    }

    /// Returns the settings that the window opens with: at its first size,
    /// and fullscreen if the operator last made it fullscreen
    fn settings(&self) -> window::Settings {
        window::Settings {
            fullscreen: self.fullscreen,
            ..window_settings(STAGE_SIZE)
        }
    }
}

impl App {
    /// Opens the stage window once the show holds a video cue, unless it
    /// is open already or the operator has closed it.  The main window then
    /// asks for the keyboard back, which the window manager may refuse, as
    /// those of Wayland do.  The stage window then keeps the keyboard, and
    /// the keys typed there act on the cue list, apart from those that
    /// would give a field of the main window the keyboard, which do nothing
    /// until the main window has it again.
    pub(in crate::front) fn staged(self) -> (Self, Task<Message>) {
        let wanted = self.stage.window.is_none()
            && !self.stage.closed
            && self.workspace.cues().any(|cue| cue.kind == Kind::Video);
        if !wanted {
            return (self, Task::none());
        }
        let (window, opened) = window::open(self.stage.settings());
        let main = self.main;
        let opening = opened.then(move |_| main.map_or_else(Task::none, window::gain_focus));
        let stage = Stage {
            window: Some(window),
            ..self.stage
        };
        (Self { stage, ..self }, opening)
    }

    /// Keeps the stage window closed once it has closed, until a video cue
    /// fires, and quits once the main window has closed
    pub(super) fn closed(self, window: window::Id) -> (Self, Task<Message>) {
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

    /// Toggles the stage window, if it is open, between fullscreen and a
    /// window, from whichever of the two it is in now, and reports which of
    /// the two the operator made it, for the window to open in again
    pub(super) fn toggle_fullscreen(&self) -> Task<Message> {
        self.stage.window.map_or_else(Task::none, |window| {
            window::mode(window).then(move |mode| {
                let fullscreen = mode != Mode::Fullscreen;
                let toggled = if fullscreen {
                    Mode::Fullscreen
                } else {
                    Mode::Windowed
                };
                window::set_mode(window, toggled)
                    .chain(Task::done(Message::StageToggled { fullscreen }))
            })
        })
    }
}
