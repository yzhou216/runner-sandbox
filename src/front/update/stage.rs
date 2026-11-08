// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stage window, which shows the pictures of the picture and video cues
//! that run
//!
//! The stage window opens once the show holds a picture or video cue, as a
//! window that the operator can move to the display of the audience, and
//! shows black while no picture shows.  Each GO that fires a picture or
//! video cue makes it fullscreen, on the display that it is on, and brings
//! it before the other windows, so that what the cue shows fills the
//! screen.  The key of the fullscreen action toggles the window between
//! fullscreen and a window, until the next such GO.  Closing it closes it
//! until a picture or video cue fires again, which opens it anew,
//! fullscreen wherever the platform puts it, while closing the main window
//! quits cinq.  Each picture that the video output hands on becomes an
//! image for iced once, as it comes due, and stays until a newer picture of
//! its cue takes its place or the cue stops.
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
    back::{CueId, Picture, Workspace},
    front::{App, Message, STAGE_SIZE, window_settings},
};

/// Stage window, with the newest picture of each picture and video cue
/// that runs
#[derive(Default)]
pub struct Stage {
    window: Option<window::Id>,
    /// Whether the operator closed the window, which keeps it closed until
    /// a picture or video cue fires
    closed: bool,
    /// Whether the window has the keyboard, which the window manager may
    /// leave with it once it opens
    keyboard: bool,
    /// Whether GO has fired a picture or video cue since the window last
    /// went fullscreen for one, which makes it go fullscreen
    cued: bool,
    /// Newest picture of each picture and video cue that runs, as iced
    /// draws it
    pictures: HashMap<CueId, image::Handle>,
    /// Seconds of the show clock until the next picture of a video cue
    /// that plays comes due, once that picture has arrived
    due: Option<f64>,
}

impl Stage {
    pub fn window(&self) -> Option<window::Id> {
        self.window
    }

    pub fn due(&self) -> Option<f64> {
        self.due
    }

    pub(in crate::front) fn with_due(self, due: Option<f64>) -> Self {
        Self { due, ..self }
    }

    pub fn is_window(&self, window: window::Id) -> bool {
        self.window == Some(window)
    }

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

    /// Returns the stage once GO has fired a picture or video cue, which
    /// opens the window again if the operator has closed it, and makes it
    /// fullscreen
    pub(super) fn cued(self) -> Self {
        Self {
            closed: false,
            cued: true,
            ..self
        }
    }

    /// Returns the settings that the window opens with: at its first size,
    /// or fullscreen when it opens for a picture or video cue that GO has
    /// fired
    fn settings(&self) -> window::Settings {
        window::Settings {
            fullscreen: self.cued,
            ..window_settings(STAGE_SIZE)
        }
    }
}

impl App {
    /// Opens the stage window once the show holds a picture or video cue,
    /// unless it is open already or the operator has closed it, and makes
    /// it fullscreen once GO has fired such a cue.  A window that opens
    /// fullscreen keeps the keyboard, while the main window asks for it
    /// back from one that opens as a window, which the window manager may
    /// refuse, as those of Wayland do.  While the stage window has the
    /// keyboard, the keys typed there act on the cue list, apart from those
    /// that would give a field of the main window the keyboard, which do
    /// nothing until the main window has it again.
    pub(in crate::front) fn staged(self) -> (Self, Task<Message>) {
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
        let wanted = !self.stage.closed && self.workspace.cues().any(|cue| cue.kind.on_stage());
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

    /// Keeps the stage window closed once it has closed, until a picture or
    /// video cue fires, and quits once the main window has closed
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
    /// window, from whichever of the two it is in now
    pub(super) fn toggle_fullscreen(&self) -> Task<Message> {
        self.stage.window.map_or_else(Task::none, |window| {
            window::mode(window).then(move |mode| {
                let toggled = if mode == Mode::Fullscreen {
                    Mode::Windowed
                } else {
                    Mode::Fullscreen
                };
                window::set_mode(window, toggled)
            })
        })
    }
}

/// Makes the stage window `window` fullscreen, on the display that it is
/// on, unless it is already, and brings it before the other windows
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
