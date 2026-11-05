// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Ticks of the clock, paced to the drawing of the windows
//!
//! winit's event loop on Windows draws a single window each time it wakes,
//! and Windows picks which, while iced asks every window to draw again
//! after each update.  Once drawing a window takes longer than the time
//! between two ticks, a tick arrives during every drawing, so that the
//! window that Windows picks first draws after every tick and another may
//! never draw at all, as winit's issue 3648 tells
//! (<https://github.com/rust-windowing/winit/issues/3648>).  Each tick here
//! waits until every window that it paces has drawn since the tick before
//! it, so that the windows take turns, at the cost of ticks that come later
//! while drawing is slow.  No tick waits longer than [`GRACE`] past its
//! time, for a window that does not draw.

#[cfg(test)]
mod tests;

use std::{
    any::TypeId,
    hash::Hash,
    pin::pin,
    time::{Duration, Instant},
};

use iced::{
    Event, Subscription,
    advanced::subscription::{self, EventStream, Hasher, Recipe},
    futures::{
        future::{self, Either},
        stream::{self, BoxStream, StreamExt},
    },
    window,
};

/// Longest that a tick waits past its time for the windows to draw
const GRACE: Duration = Duration::from_millis(100);

/// Returns ticks of the clock, the first `wait` from now and each later one
/// `wait` after the one before, each of which waits until every window of
/// `windows` has drawn since the one before, or until it is [`GRACE`] late
pub fn every(wait: Duration, windows: Vec<window::Id>) -> Subscription<Instant> {
    subscription::from_recipe(Every { wait, windows })
}

/// Recipe of [`every`]
struct Every {
    wait: Duration,
    windows: Vec<window::Id>,
}

impl Recipe for Every {
    type Output = Instant;

    fn hash(&self, state: &mut Hasher) {
        TypeId::of::<Self>().hash(state);
        self.wait.hash(state);
        self.windows.hash(state);
    }

    fn stream(self: Box<Self>, input: EventStream) -> BoxStream<'static, Instant> {
        let draws = input.filter_map(|event| future::ready(drawn(event)));
        let pace = Pace::new(self.wait, self.windows, Instant::now());
        // Each step either ticks or counts a window that has drawn
        stream::unfold((pace, draws), |(pace, mut draws)| async move {
            let release = tokio::time::sleep_until(pace.release().into());
            match future::select(pin!(release), draws.next()).await {
                Either::Left(_) => {
                    let now = Instant::now();
                    Some((Some(now), (pace.ticked(now), draws)))
                }
                Either::Right((Some(window), _)) => Some((None, (pace.drawn(window), draws))),
                // Events end along with the subscription
                Either::Right((None, _)) => None,
            }
        })
        .filter_map(future::ready)
        .boxed()
    }
}

/// Returns the window that `event` tells has drawn, if it tells of one
fn drawn(event: subscription::Event) -> Option<window::Id> {
    match event {
        subscription::Event::Interaction {
            window,
            event: Event::Window(window::Event::RedrawRequested(_)),
            ..
        } => Some(window),
        _ => None,
    }
}

/// Next tick of a pace, and the windows that have yet to draw before it
struct Pace {
    /// Time from each tick to the next
    wait: Duration,
    /// Windows that each tick waits for
    windows: Vec<window::Id>,
    /// Time that the next tick comes due at
    due: Instant,
    /// Windows that have not drawn since the last tick
    undrawn: Vec<window::Id>,
}

impl Pace {
    /// Returns a pace of `windows` whose first tick comes due `wait` after
    /// `now`
    fn new(wait: Duration, windows: Vec<window::Id>, now: Instant) -> Self {
        Self {
            wait,
            undrawn: windows.clone(),
            windows,
            due: now + wait,
        }
    }

    /// Returns the time that the next tick goes at, as far as the windows
    /// have drawn: its due time once every window has drawn, and until then
    /// [`GRACE`] later
    fn release(&self) -> Instant {
        if self.undrawn.is_empty() {
            self.due
        } else {
            self.due + GRACE
        }
    }

    /// Returns the pace once `window` has drawn
    fn drawn(self, window: window::Id) -> Self {
        let undrawn = self
            .undrawn
            .into_iter()
            .filter(|undrawn| *undrawn != window)
            .collect();
        Self { undrawn, ..self }
    }

    /// Returns the pace after a tick at `now`, whose next tick comes due
    /// `wait` later and waits for every window to draw again
    fn ticked(self, now: Instant) -> Self {
        Self::new(self.wait, self.windows, now)
    }
}
