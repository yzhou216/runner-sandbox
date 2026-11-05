// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Subscriptions, which turn keys, clicks, the clock, the pictures of video
//! cues as they come due, windows that gain the keyboard, change size, or
//! close, and changes of the configuration file into messages
//!
//! On Windows, the ticks of the clock wait for every window to draw, as
//! [`pace`] tells.

mod pace;

use std::time::Duration;

use iced::{Event, Size, Subscription, event, keyboard, mouse, time, window};
use itertools::chain;

use super::keys;
use crate::{
    back::TICK,
    front::{App, Message, keymap},
};

/// Interval at which the files that could not be played are tried again
const RECHECK: Duration = Duration::from_secs(2);

/// Shortest wait for the next picture of a video cue, since a timer cannot
/// wait for no time at all
const SOONEST: Duration = Duration::from_millis(1);

impl App {
    /// Listens for keys and clicks in either window, windows that gain the
    /// keyboard, change size, or close, and changes of the configuration,
    /// for clock ticks while any cue is running, a tick after the last, or
    /// sooner when the next picture of a video cue comes due before then,
    /// so that it shows then, and for the time to try again the files that
    /// could not be played
    pub(super) fn subscription(&self) -> Subscription<Message> {
        let tick = self.running().then_some(TICK);
        let picture = self
            .stage
            .due()
            .filter(|_| self.stage.window().is_some() && self.moving_pictures())
            .map(|due| Duration::from_secs_f64(due).max(SOONEST));
        let ticks = chain!(tick, picture)
            .min()
            .map(|wait| pace::after(self.last_tick, wait, self.paced_windows()).map(Message::Tick));
        let rechecks = (!self.rechecking && self.workspace.failed().next().is_some())
            .then(|| time::every(RECHECK).map(|_| Message::Recheck));
        let always = [
            keyboard::listen().filter_map(keys::key_press),
            event::listen_with(clicked),
            event::listen_with(focused),
            window::resize_events().map(|(window, size)| Message::Resized(window, size)),
            window::close_events().map(Message::Closed),
        ];
        let reconfigured = self.config.source.file().map(|file| {
            Subscription::run_with(file.to_owned(), |file| keymap::config::watch(file.clone()))
                .map(Message::Reconfigured)
        });
        Subscription::batch(chain!(always, ticks, rechecks, reconfigured))
    }

    /// Returns the windows whose drawing the ticks of the clock wait for:
    /// on Windows, those that iced draws, and elsewhere none, since only
    /// winit's event loop on Windows lets one window keep another from
    /// drawing
    fn paced_windows(&self) -> Vec<window::Id> {
        if cfg!(windows) {
            self.drawn_windows()
        } else {
            Vec::new()
        }
    }

    /// Returns the windows that iced draws, which are those open and not
    /// minimized
    pub(super) fn drawn_windows(&self) -> Vec<window::Id> {
        [self.main, self.stage.window()]
            .into_iter()
            .flatten()
            .filter(|window| !self.minimized.contains(window))
            .collect()
    }

    /// Returns the state once `window` has changed to `size`, which counts
    /// it as minimized while it has no size, as minimized windows have on
    /// Windows
    pub(super) fn resized(self, window: window::Id, size: Size) -> Self {
        self.with_minimized(window, size.width == 0.0 || size.height == 0.0)
    }

    /// Returns the state with `window` among the minimized windows, or
    /// apart from them
    pub(super) fn with_minimized(self, window: window::Id, minimized: bool) -> Self {
        let minimized = self
            .minimized
            .into_iter()
            .filter(|minimized| *minimized != window)
            .chain(minimized.then_some(window))
            .collect();
        Self { minimized, ..self }
    }
}

/// Reports a press of any mouse button, captured by a widget or not, so
/// that a click drops a command typed halfway
fn clicked(event: Event, _status: event::Status, _window: window::Id) -> Option<Message> {
    matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))).then_some(Message::Clicked)
}

/// Reports the window that gains the keyboard
fn focused(event: Event, _status: event::Status, id: window::Id) -> Option<Message> {
    matches!(event, Event::Window(window::Event::Focused)).then_some(Message::WindowFocused(id))
}
