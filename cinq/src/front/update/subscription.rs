// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Subscriptions, which turn keys, clicks, the clock, the pictures of video
//! cues as they come due, windows that gain the keyboard, change size, or
//! close, and changes of the configuration file into messages

use std::time::Duration;

use iced::{Event, Subscription, event, keyboard, mouse, time, window};
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
    /// for clock ticks while any cue is running, for the time at which the
    /// next picture of a video cue comes due, so that it shows then whether
    /// or not a tick falls there, and for the time to try again the files
    /// that could not be played
    pub(super) fn subscription(&self) -> Subscription<Message> {
        let ticks = self.running().then(|| time::every(TICK).map(Message::Tick));
        let picture = self
            .stage
            .due()
            .filter(|_| self.stage.window().is_some() && self.moving_pictures())
            .map(|due| time::every(Duration::from_secs_f64(due).max(SOONEST)).map(Message::Tick));
        let rechecks = (!self.rechecking && self.workspace.failed().next().is_some())
            .then(|| time::every(RECHECK).map(|_| Message::Recheck));
        let always = [
            keyboard::listen().filter_map(keys::key_press),
            event::listen_with(clicked),
            event::listen_with(focused),
            window::resize_events().map(|(window, _)| Message::Resized(window)),
            window::close_events().map(Message::Closed),
        ];
        let reconfigured = self.config.source.file().map(|file| {
            Subscription::run_with(file.to_owned(), |file| keymap::config::watch(file.clone()))
                .map(Message::Reconfigured)
        });
        Subscription::batch(chain!(always, ticks, picture, rechecks, reconfigured))
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
