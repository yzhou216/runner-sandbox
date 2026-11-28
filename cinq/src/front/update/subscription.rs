// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Subscriptions, which turn events into messages
//!
//! On Windows, the ticks of the clock wait for every window to draw, as
//! [`pace`] tells.

mod pace;
#[cfg(test)]
mod tests;

use std::time::Duration;

use iced::{Event, Size, Subscription, Task, event, keyboard, mouse, time, window};
use itertools::chain;

use super::keys;
use crate::{
    back::{Heard, TICK, Watch, osc_listen, watch},
    front::{App, Message, config},
};

/// Interval at which the files of the show are looked at, for a change
const RECHECK: Duration = Duration::from_secs(2);

/// How long the search line tells that no row matches
const NO_MATCH: Duration = Duration::from_secs(2);

/// Shortest wait for the next picture of a video cue, since a timer cannot
/// wait for no time at all
const SOONEST: Duration = Duration::from_millis(1);

impl App {
    pub(super) fn subscription(&self) -> Subscription<Message> {
        let drawing = self.unmeasured().then(|| event::listen_with(drawn));
        let settling = self
            .search
            .as_ref()
            .is_some_and(|search| search.warned)
            .then(|| time::every(NO_MATCH).map(|_| Message::SearchSettled));
        let network = self
            .network_checks
            .map(|every| time::every(every).map(|_| Message::NetworkChecked));
        Subscription::batch(chain!(
            Self::always(self.main),
            drawing,
            self.ticks(),
            self.rechecks(),
            settling,
            network,
            self.reconfigured(),
            self.listening()
        ))
    }

    /// Returns the subscriptions that run from start to end: the keys, the
    /// mouse, and the windows
    fn always(main: Option<window::Id>) -> [Subscription<Message>; 6] {
        [
            keyboard::listen().filter_map(keys::key_press),
            event::listen_with(clicked)
                .with(main)
                .filter_map(|(main, (window, message))| in_window(main, window, message)),
            event::listen_with(side_pressed),
            event::listen_with(focused),
            window::resize_events().map(|(window, size)| Message::Resized(window, size)),
            window::close_events().map(Message::Closed),
        ]
    }

    /// Returns the clock, which ticks when a cue runs or a picture is due
    fn ticks(&self) -> Option<Subscription<Message>> {
        let tick = self.running().then_some(TICK);
        let picture = self
            .stage
            .due()
            .filter(|_| self.stage.window().is_some() && self.moving_pictures())
            .map(|due| Duration::from_secs_f64(due).max(SOONEST));
        chain!(tick, picture)
            .min()
            .map(|wait| pace::after(self.last_tick, wait, self.paced_windows()).map(Message::Tick))
    }

    /// Returns the watch over the files that cues have read, if any
    fn rechecks(&self) -> Option<Subscription<Message>> {
        // The files that cues have read identify the watch, so that a file
        // read anew starts the next one
        Some(Watch::all(&self.workspace, self.workspace.to_recheck()))
            .filter(|files| !files.is_empty())
            .map(|files| {
                Subscription::run_with(files, |files| watch(files.clone(), RECHECK))
                    .map(|()| Message::Recheck)
            })
    }

    /// Returns the watch over the file of the configuration, if it has one
    fn reconfigured(&self) -> Option<Subscription<Message>> {
        self.config.source.file().map(|file| {
            Subscription::run_with(file.to_owned(), |file| {
                config::watch(file.clone(), config::Errors::standard())
            })
            .map(|config| Message::Reconfigured(Box::new(config)))
        })
    }

    /// Returns the listener for incoming OSC, if the settings give an address
    fn listening(&self) -> Option<Subscription<Message>> {
        // Another address ends this subscription and starts the next, which
        // iced does by the changed data
        self.config.osc_listen.map(|address| {
            Subscription::run_with(address, |address| osc_listen(*address)).map(|heard| match heard
            {
                Heard::Command(from, remote) => Message::Remote(from, remote),
                Heard::Ignored(line) => Message::OscIgnored(line),
                Heard::Failed(problem) => Message::OscFailed(problem),
                Heard::Listening => Message::OscListening,
            })
        })
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
    pub(super) fn resized(self, window: window::Id, size: Size) -> (Self, Task<Message>) {
        let app = self.with_minimized(window, size.width == 0.0 || size.height == 0.0);
        if app.main != Some(window) {
            return (app, Task::none());
        }
        // Rows of a half page that a count set, which Vim resets as the window
        // changes size, and the measures with them, which the task measures
        // anew so that the first page key finds one kept.  The labels of hint
        // mode would no longer lie on their places.
        Self {
            scroll: None,
            ..app.without_hints()
        }
        .with_window_height(size.height)
        .revealed()
    }

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
/// that a click drops a command typed halfway, and the end of a press or a
/// turn of the wheel, after which the cue list may have scrolled and the labels
/// of hint mode no longer lie on their places
fn clicked(
    event: Event,
    _status: event::Status,
    window: window::Id,
) -> Option<(window::Id, Message)> {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(_)) => Some((window, Message::Clicked)),
        Event::Mouse(mouse::Event::ButtonReleased(_) | mouse::Event::WheelScrolled { .. }) => {
            Some((window, Message::Remeasure))
        }
        _ => None,
    }
}

/// Keeps `message` only for an event in the main window, since a click or a
/// turn of the wheel in the stage window touches neither the search line, nor
/// hint mode, nor the rows
fn in_window(main: Option<window::Id>, window: window::Id, message: Message) -> Option<Message> {
    (main == Some(window)).then_some(message)
}

fn drawn(event: Event, _status: event::Status, window: window::Id) -> Option<Message> {
    matches!(event, Event::Window(window::Event::RedrawRequested(_)))
        .then_some(Message::Drawn(window))
}

fn focused(event: Event, _status: event::Status, id: window::Id) -> Option<Message> {
    matches!(event, Event::Window(window::Event::Focused)).then_some(Message::WindowFocused(id))
}

fn side_pressed(event: Event, _status: event::Status, window: window::Id) -> Option<Message> {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(
            button @ (mouse::Button::Back | mouse::Button::Forward),
        )) => Some(Message::SidePressed(window, button)),
        _ => None,
    }
}
