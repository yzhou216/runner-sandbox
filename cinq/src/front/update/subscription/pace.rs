// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Ticks of the clock, paced to the drawing of the windows
//!
//! winit's event loop on Windows draws one window each time it wakes, the
//! one that Windows picks, and updates with the messages of the wake only
//! after that drawing, while iced asks every window to draw again after
//! each update.  Once a drawing takes longer than the time between two
//! ticks, every wake brings a tick, whose update sends the window that
//! Windows picks back to draw again, so that another window may never draw
//! at all, as winit's issue 3648 tells
//! (<https://github.com/rust-windowing/winit/issues/3648>).  Each tick here
//! waits until every window that it paces has drawn since the update before
//! it, so that the windows take turns, at the cost of fewer ticks while
//! drawing is slow.  Each pace starts with an update and gives a single
//! tick before the update of that tick starts the next pace, since a tick
//! more, which a late update would let through, would bring an update
//! before the windows draw.

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

/// Longest that a tick waits for the windows that have yet to draw, past
/// the first drawing since the update before it.  A tick that comes then
/// leaves the windows their turns, since winit updates with it only after
/// the next wake, which draws a window that has yet to draw.
const GRACE: Duration = Duration::from_millis(50);

/// Longest that a tick waits for the first drawing since the update before
/// it, which only windows that never draw make it wait in full.  A tick
/// that came before that drawing would bring its update right after it,
/// which would send the window that drew back to draw first.
const STALL: Duration = Duration::from_secs(1);

/// Returns a tick of the clock `wait` after `last`, the time of the last
/// tick, or `wait` from now without one, which waits for the windows of
/// `windows` to draw, as far as [`GRACE`] and [`STALL`] allow.  The update
/// of that tick asks for the next.
pub fn after(
    last: Option<Instant>,
    wait: Duration,
    windows: Vec<window::Id>,
) -> Subscription<Instant> {
    subscription::from_recipe(Ticks {
        last,
        wait,
        windows,
    })
}

/// Recipe of [`after`]
struct Ticks {
    last: Option<Instant>,
    wait: Duration,
    windows: Vec<window::Id>,
}

impl Recipe for Ticks {
    type Output = Instant;

    fn hash(&self, state: &mut Hasher) {
        TypeId::of::<Self>().hash(state);
        self.last.hash(state);
        self.wait.hash(state);
        self.windows.hash(state);
    }

    fn stream(self: Box<Self>, input: EventStream) -> BoxStream<'static, Instant> {
        let draws = input.filter_map(|event| future::ready(drawn(event)));
        // Stream starts during the update that asks for it
        let now = Instant::now();
        let due = self.last.unwrap_or(now) + self.wait;
        let pace = Pace::new(self.windows, now, due);
        // Each step either ticks or counts a window that has drawn
        stream::unfold((pace, draws), |(pace, mut draws)| async move {
            let release = tokio::time::sleep_until(pace.release().into());
            match future::select(pin!(release), draws.next()).await {
                Either::Left(_) => {
                    let now = Instant::now();
                    Some((Some(now), (pace.ticked(now), draws)))
                }
                Either::Right((Some(window), _)) => {
                    Some((None, (pace.drawn(window, Instant::now()), draws)))
                }
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
    /// Time from which the windows count as drawn: the update that started
    /// the pace, and then its last tick
    since: Instant,
    /// Time that the next tick comes due at
    due: Instant,
    /// Windows that have not drawn since then
    undrawn: Vec<window::Id>,
    /// Time of the first drawing since then
    first: Option<Instant>,
}

impl Pace {
    /// Returns a pace whose next tick comes due at `due` and waits for
    /// `windows` to draw from `since`
    fn new(windows: Vec<window::Id>, since: Instant, due: Instant) -> Self {
        Self {
            since,
            due,
            undrawn: windows,
            first: None,
        }
    }

    /// Returns the time that the next tick goes at, as far as the windows
    /// have drawn: its due time once every window has drawn, and until then
    /// no sooner than [`GRACE`] past the first drawing, or [`STALL`] past
    /// the start without one
    fn release(&self) -> Instant {
        if self.undrawn.is_empty() {
            self.due
        } else {
            let wait = self.first.map_or(self.since + STALL, |first| first + GRACE);
            self.due.max(wait)
        }
    }

    /// Returns the pace once `window` has drawn at `now`
    fn drawn(self, window: window::Id, now: Instant) -> Self {
        let undrawn = self
            .undrawn
            .into_iter()
            .filter(|undrawn| *undrawn != window)
            .collect();
        Self {
            undrawn,
            first: self.first.or(Some(now)),
            ..self
        }
    }

    /// Returns the pace after a tick at `now`, which the update of that tick
    /// replaces with a pace of its own.  Until then, the next tick waits
    /// [`STALL`], whether or not the windows draw, so that ticks cannot pile
    /// up while the update is late, and the clock still goes on should no
    /// update come.
    fn ticked(self, now: Instant) -> Self {
        Self::new(Vec::new(), now, now + STALL)
    }
}
