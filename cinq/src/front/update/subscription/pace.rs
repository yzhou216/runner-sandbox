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
//! tick before the update of that tick starts the next pace.

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

/// Longest that a tick waits for the windows to draw, past the first
/// drawing since the update before it, or past that update while no window
/// draws.  A tick that comes during a drawing leaves the windows their
/// turns, since winit updates with it only after the next wake, which draws
/// a window that has yet to draw.
const GRACE: Duration = Duration::from_millis(50);

/// Returns ticks of the clock, the first `wait` after `last`, the time of
/// the last tick, or `wait` from now without one, and each later one `wait`
/// after the one before, each of which waits for the windows of `windows`
/// to draw, as far as [`GRACE`] allows
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
        let elapsed = || crate::START.elapsed().as_secs_f64();
        eprintln!(
            "{:>8.3} pace of {:?}, wait {:?}, due in {:?}",
            elapsed(),
            self.windows,
            self.wait,
            due.saturating_duration_since(now)
        );
        let pace = Pace::new(self.wait, self.windows, now, due);
        // Each step either ticks or counts a window that has drawn
        stream::unfold((pace, draws), move |(pace, mut draws)| async move {
            let release = tokio::time::sleep_until(pace.release().into());
            match future::select(pin!(release), draws.next()).await {
                Either::Left(_) => {
                    let now = Instant::now();
                    eprintln!("{:>8.3} tick, undrawn {:?}", elapsed(), pace.undrawn);
                    Some((Some(now), (pace.ticked(now), draws)))
                }
                Either::Right((Some(window), _)) => {
                    eprintln!("{:>8.3} drawn {window:?}", elapsed());
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
    /// Time from each tick to the next
    wait: Duration,
    /// Windows that each tick waits for
    windows: Vec<window::Id>,
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
    /// Returns a pace of `windows`, which count as drawn from `since`, whose
    /// next tick comes due at `due`
    fn new(wait: Duration, windows: Vec<window::Id>, since: Instant, due: Instant) -> Self {
        Self {
            wait,
            undrawn: windows.clone(),
            windows,
            since,
            due,
            first: None,
        }
    }

    /// Returns the time that the next tick goes at, as far as the windows
    /// have drawn: its due time once every window has drawn, and until then
    /// no sooner than [`GRACE`] past the first drawing, or past the start
    /// without one
    fn release(&self) -> Instant {
        if self.undrawn.is_empty() {
            self.due
        } else {
            self.due.max(self.first.unwrap_or(self.since) + GRACE)
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

    /// Returns the pace after a tick at `now`, for want of an update that
    /// starts a pace in its place, whose next tick comes due `wait` later
    /// and waits for the windows to draw again
    fn ticked(self, now: Instant) -> Self {
        Self::new(self.wait, self.windows, now, now + self.wait)
    }
}
