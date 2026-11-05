// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the ticks that wait for the windows to draw

use std::time::{Duration, Instant};

use iced::window;

use super::{GRACE, Pace};

/// Time between two ticks of the paces under test
const WAIT: Duration = Duration::from_millis(25);

#[test]
fn ticks_of_no_windows_go_on_time() {
    let now = Instant::now();
    assert_eq!(Pace::new(WAIT, Vec::new(), now).release(), now + WAIT);
}

#[test]
fn tick_waits_for_every_window_to_draw_but_no_longer_than_its_grace() {
    let (main, stage) = (window::Id::unique(), window::Id::unique());
    let now = Instant::now();
    let pace = Pace::new(WAIT, vec![main, stage], now);
    assert_eq!(pace.release(), now + WAIT + GRACE);
    let pace = pace.drawn(stage);
    assert_eq!(pace.release(), now + WAIT + GRACE);
    assert_eq!(pace.drawn(main).release(), now + WAIT);
}

#[test]
fn each_tick_waits_for_the_windows_to_draw_again() {
    let main = window::Id::unique();
    let now = Instant::now();
    let later = now + 3 * WAIT;
    let pace = Pace::new(WAIT, vec![main], now).drawn(main).ticked(later);
    assert_eq!(pace.release(), later + WAIT + GRACE);
}
