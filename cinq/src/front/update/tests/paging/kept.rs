// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the measures that the app keeps of the cue list and of the help
//! page, so that page keys pressed in a row each move a full step

use iced::{Size, window};

use super::{
    super::{history::cue, show, windowed},
    typed,
};
use crate::{
    back::{CueId, Place},
    front::{
        App, MAIN_SIZE, Message,
        message::Page,
        update::{
            Measured,
            rows::paging::{help_offset, viewport::Shape},
        },
        view::line_height,
    },
};

/// Returns the state once the list of twelve cues has been measured, as a
/// view of five rows of twenty pixels
fn measured_list(app: App) -> App {
    let (rows, count) = (app.rows(), app.current().cues.len());
    let shape = Shape {
        view: 100.0,
        content: count as f32 * 20.0,
        scrolled: 0.0,
    };
    let serial = app.measures.serial();
    app.handled(Message::Measured(Measured::Rows {
        serial,
        rows,
        count,
        shape,
    }))
    .0
}

/// Returns the selection and the tasks started after each of `times` presses of
/// `key` with Control, and the state after the last
fn presses(app: App, key: &str, times: usize) -> (App, Vec<(Option<CueId>, usize)>) {
    (0..times).fold((app, Vec::new()), |(app, seen), _| {
        let (app, started) = typed(app, key, &[]);
        let seen = [seen, vec![(app.selected, started)]].concat();
        (app, seen)
    })
}

#[test]
fn pages_and_half_pages_pressed_in_a_row_each_move_a_full_step_by_the_measure_kept() {
    // Five rows fit, a whole page is three, and the selection starts at the top
    let app = measured_list(show(Some(CueId(1))));
    // First row in view after each page, then the last cue at the end
    let (third, seen) = presses(app, "f", 3);
    assert_eq!(seen, [(cue(4), 2), (cue(7), 2), (cue(12), 2)]);
    // Page up lands on the last row in view
    let (up, _) = typed(third, "b", &[]);
    assert_eq!(up.selected, Some(CueId(9)));
    // Half pages keep their row on the screen
    let (_, seen) = presses(measured_list(show(Some(CueId(1)))), "d", 3);
    assert_eq!(seen, [(cue(3), 2), (cue(5), 2), (cue(7), 2)]);
}

#[test]
fn a_move_that_scrolls_or_a_change_in_the_list_drops_the_measure_kept() {
    let moved = |app: App| {
        let before = app.selected;
        typed(app, "d", &[]).0.selected != before
    };
    let kept = || measured_list(show(Some(CueId(1))));
    assert!(moved(kept()));
    // Without a measure the key waits for one, and moves nothing yet
    let (stepped, _) = kept().handled(Message::Step(1));
    assert!(!moved(stepped));
    let (wheeled, _) = kept().handled(Message::Remeasure);
    assert!(!moved(wheeled));
    let (added, _) = kept().handled(Message::Add(Place::After, 1));
    assert!(!moved(added));
}

#[test]
fn help_page_scrolls_by_its_height_in_view_and_by_the_height_of_the_window_before_a_measure() {
    let by = |page, count, scroll, view| help_offset(page, count, scroll, view).y;
    let line = line_height();
    assert_eq!(by(Page::HalfDown, None, None, 400.0), 200.0);
    assert_eq!(by(Page::HalfUp, None, Some(3), 400.0), -3.0 * line);
    assert_eq!(
        by(Page::WholeDown, Some(2), None, 400.0),
        2.0 * (400.0 - 2.0 * line)
    );
    // The height of the window stands in where nothing is measured, so that
    // the first key after the page opens scrolls it
    let window = MAIN_SIZE.height;
    assert!(by(Page::WholeDown, None, None, window) > 0.0);
    let (window, main) = windowed(App::default());
    let measured = window.with_help_measure(300.0);
    assert_eq!(measured.help_heights(), (Some(300.0), None));
    // Change of size takes the measure away and keeps the height
    let (resized, _) = measured.handled(Message::Resized(main, Size::new(800.0, 600.0)));
    assert_eq!(resized.help_heights(), (None, Some(600.0)));
}

#[test]
fn resizing_the_main_window_measures_the_list_so_that_page_keys_find_a_measure() {
    let size = Size::new(800.0, 600.0);
    let (app, _) = windowed(show(Some(CueId(1))));
    let (other, task) = app.handled(Message::Resized(window::Id::unique(), size));
    assert_eq!((other.unmeasured(), task.units()), (true, 0));
    let (app, main) = windowed(show(Some(CueId(1))));
    let (resized, task) = app.handled(Message::Resized(main, size));
    assert_eq!(task.units(), 2);
    assert!(resized.unmeasured());
    // Kept measure stays until something makes it out of date
    let kept = measured_list(resized);
    assert!(!kept.unmeasured());
}

#[test]
fn a_drawing_of_the_main_window_measures_the_list_while_no_measure_is_kept() {
    let drawn = |app: App, window| app.handled(Message::Drawn(window)).1.units();
    let (app, main) = windowed(show(Some(CueId(1))));
    assert_eq!(drawn(app, main), 1);
    let (app, _) = windowed(show(Some(CueId(1))));
    assert_eq!(drawn(app, window::Id::unique()), 0);
    let (app, main) = windowed(show(Some(CueId(1))));
    assert_eq!(drawn(measured_list(app), main), 0);
}

#[test]
fn three_pages_pressed_in_a_row_after_a_resize_each_move_a_full_step() {
    let (app, main) = windowed(show(Some(CueId(1))));
    let (resized, _) = app.handled(Message::Resized(main, Size::new(800.0, 600.0)));
    let (_, seen) = presses(measured_list(resized), "f", 3);
    assert_eq!(seen, [(cue(4), 2), (cue(7), 2), (cue(12), 2)]);
}
