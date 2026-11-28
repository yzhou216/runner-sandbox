// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stage window, which shows the pictures of the picture and video cues
//! that run
//!
//! The window opens once the show holds a picture or video cue, so that the
//! operator can move it to the display of the audience, and shows black
//! while no picture shows.  Each GO that fires such a cue, itself or through
//! a start cue, makes it fullscreen on the display that it is on and raises
//! it, until the key of fullscreen toggles it.  Closing it keeps it closed
//! until such a cue fires again or the key of fullscreen is typed, which
//! opens it anew, fullscreen wherever the platform puts it, while closing
//! the main window quits cinq.  Each picture that the video output hands on
//! becomes an image for iced once, as it comes due, and stays until a newer
//! picture of its cue takes its place or the cue stops.
//!
//! Each picture costs a whole update, as each tick of the clock does, which
//! builds the view of the main window again along with that of the stage.
//! On an Intel Core i7-13700H, a picture takes the thread of the interface
//! about 2 ms whether the show holds three cues or 300, while the 40 ticks a
//! second of the clock take 2% of its time with three cues and 10% with 300.

mod lifecycle;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use iced::{widget::image, window};

use crate::{
    back::{CueId, Picture, Workspace},
    front::{STAGE_SIZE, window_settings},
};

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
    pub(super) fn window(&self) -> Option<window::Id> {
        self.window
    }

    pub(super) fn due(&self) -> Option<f64> {
        self.due
    }

    pub(super) fn with_due(self, due: Option<f64>) -> Self {
        Self { due, ..self }
    }

    pub(super) fn is_window(&self, window: window::Id) -> bool {
        self.window == Some(window)
    }

    pub fn has_keyboard(&self) -> bool {
        self.keyboard
    }

    pub(super) fn with_keyboard_in(self, window: window::Id) -> Self {
        Self {
            keyboard: self.is_window(window),
            ..self
        }
    }

    /// Returns whether the window is closed while a cue shows on it, which
    /// leaves the audience without the picture that the cue says it shows
    pub fn missing(&self, workspace: &Workspace) -> bool {
        self.closed
            && workspace
                .active()
                .iter()
                .any(|active| active.shows_picture())
    }

    pub fn picture(&self, cue: CueId) -> Option<&image::Handle> {
        self.pictures.get(&cue)
    }

    pub(super) fn with_pictures(
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

    pub(super) fn cued(self) -> Self {
        Self {
            closed: false,
            cued: true,
            ..self
        }
    }

    fn settings(&self) -> window::Settings {
        window::Settings {
            fullscreen: self.cued,
            ..window_settings(STAGE_SIZE)
        }
    }
}

#[cfg(test)]
impl Stage {
    pub fn closed() -> Self {
        Self {
            closed: true,
            ..Self::default()
        }
    }

    pub fn holding_keyboard() -> Self {
        Self {
            keyboard: true,
            ..Self::default()
        }
    }
}
