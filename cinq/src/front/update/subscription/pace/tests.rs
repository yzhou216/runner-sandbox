// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the ticks that wait for the windows to draw

use std::time::{Duration, Instant};

use iced::window;

use super::{GRACE, Pace, STALL};

/// Time between two ticks of the paces under test
const WAIT: Duration = Duration::from_millis(25);

/// Returns a pace of `windows` that starts at `start`, with its tick due
/// [`WAIT`] later
fn pace(windows: Vec<window::Id>, start: Instant) -> Pace {
    Pace::new(windows, start, start + WAIT)
}

#[test]
fn ticks_of_no_windows_go_on_time() {
    let start = Instant::now();
    assert_eq!(pace(Vec::new(), start).release(), start + WAIT);
}

#[test]
fn tick_waits_for_a_first_drawing_no_longer_than_a_stall() {
    let start = Instant::now();
    let release = pace(vec![window::Id::unique()], start).release();
    assert_eq!(release, start + STALL);
}

#[test]
fn tick_waits_for_every_window_until_a_grace_past_the_first_drawing() {
    let (main, stage) = (window::Id::unique(), window::Id::unique());
    let start = Instant::now();
    let drawn = start + 2 * WAIT;
    let staged = pace(vec![main, stage], start).drawn(stage, drawn);
    assert_eq!(staged.release(), drawn + GRACE);
    let again = staged.drawn(stage, drawn + WAIT);
    assert_eq!(again.release(), drawn + GRACE);
    assert_eq!(again.drawn(main, drawn + WAIT).release(), start + WAIT);
}

#[test]
fn overdue_tick_still_waits_for_a_window_to_draw_since_the_update() {
    let (main, stage) = (window::Id::unique(), window::Id::unique());
    let due = Instant::now();
    let start = due + 3 * WAIT;
    let late = Pace::new(vec![main, stage], start, due);
    assert_eq!(late.release(), start + STALL);
    let drawn = start + WAIT;
    let drawn_once = late.drawn(main, drawn);
    assert_eq!(drawn_once.release(), drawn + GRACE);
    assert_eq!(drawn_once.drawn(stage, drawn).release(), due);
}

#[test]
fn tick_that_no_update_follows_holds_the_next_for_a_stall() {
    let main = window::Id::unique();
    let start = Instant::now();
    let later = start + 3 * WAIT;
    let ticked = pace(vec![main], start)
        .drawn(main, start + WAIT)
        .ticked(later);
    assert_eq!(ticked.release(), later + STALL);
    assert_eq!(ticked.drawn(main, later + WAIT).release(), later + STALL);
}
