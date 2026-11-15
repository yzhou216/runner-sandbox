// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the measures that the app keeps of the cue list and of the help
//! page, so that page keys pressed in a row each move a full step

use iced::{Size, window};

use super::{super::show, typed};
use crate::{
    back::{CueId, Place},
    front::{
        App, MAIN_SIZE, Message,
        message::Page,
        update::{
            Measured,
            paging::{help_offset, viewport::Shape},
        },
        view::line_height,
    },
};

/// Returns the state once the list of twelve cues has been measured, as a
/// view of five rows of twenty pixels
fn measured_list(app: App) -> App {
    let (list, cues) = (app.list, app.current().cues.len());
    let shape = Shape {
        view: 100.0,
        content: cues as f32 * 20.0,
        scrolled: 0.0,
    };
    let serial = app.measures.serial();
    app.handled(Message::Measured(Measured::Cues {
        serial,
        list,
        cues,
        shape,
    }))
    .0
}

#[test]
fn pages_pressed_in_a_row_each_move_a_full_step_by_the_measure_kept() {
    // Five rows fit, a whole page is three, and the selection starts at the top
    let app = measured_list(show(Some(CueId(1))));
    let pressed = |app| {
        let (app, started) = typed(app, "f", &[]);
        assert_eq!(started, 2);
        app
    };
    // First row in view after each page, then the last cue at the end
    let first = pressed(app);
    assert_eq!(first.selected, Some(CueId(4)));
    let second = pressed(first);
    assert_eq!(second.selected, Some(CueId(7)));
    let third = pressed(second);
    assert_eq!(third.selected, Some(CueId(12)));
    // Page up lands on the last row in view
    let (up, _) = typed(third, "b", &[]);
    assert_eq!(up.selected, Some(CueId(9)));
}

#[test]
fn half_pages_pressed_in_a_row_keep_their_row_on_the_screen() {
    let app = measured_list(show(Some(CueId(1))));
    let (once, _) = typed(app, "d", &[]);
    assert_eq!(once.selected, Some(CueId(3)));
    let (twice, _) = typed(once, "d", &[]);
    assert_eq!(twice.selected, Some(CueId(5)));
    let (thrice, _) = typed(twice, "d", &[]);
    assert_eq!(thrice.selected, Some(CueId(7)));
}

#[test]
fn a_move_that_scrolls_or_a_change_in_the_list_drops_the_measure_kept() {
    let moved = |app: App| {
        let before = app.selected;
        (typed(app, "d", &[]).0.selected != before) as usize
    };
    let kept = || measured_list(show(Some(CueId(1))));
    assert_eq!(moved(kept()), 1);
    // Without a measure the key waits for one, and moves nothing yet
    let (stepped, _) = kept().handled(Message::Step(1));
    assert_eq!(moved(stepped), 0);
    let (wheeled, _) = kept().handled(Message::Remeasure);
    assert_eq!(moved(wheeled), 0);
    let (added, _) = kept().handled(Message::Add(Place::After, 1));
    assert_eq!(moved(added), 0);
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
    let main = window::Id::unique();
    let measured = App {
        main: Some(main),
        ..App::default()
    }
    .with_help_measure(300.0);
    assert_eq!(measured.help_heights(), (Some(300.0), None));
    // Change of size takes the measure away and keeps the height
    let (resized, _) = measured.handled(Message::Resized(main, Size::new(800.0, 600.0)));
    assert_eq!(resized.help_heights(), (None, Some(600.0)));
}

/// Returns the state of the twelve cues with a main window, and that window
fn windowed() -> (App, window::Id) {
    let main = window::Id::unique();
    let app = App {
        main: Some(main),
        ..show(Some(CueId(1)))
    };
    (app, main)
}

#[test]
fn resizing_the_main_window_measures_the_list_so_that_page_keys_find_a_measure() {
    let size = Size::new(800.0, 600.0);
    let (app, _) = windowed();
    let (other, task) = app.handled(Message::Resized(window::Id::unique(), size));
    assert_eq!((other.unmeasured(), task.units()), (true, 0));
    let (app, main) = windowed();
    let (resized, task) = app.handled(Message::Resized(main, size));
    assert_eq!(task.units(), 2);
    assert!(resized.unmeasured());
    // The kept measure stays until something makes it out of date
    let kept = measured_list(resized);
    assert!(!kept.unmeasured());
}

#[test]
fn a_drawing_of_the_main_window_measures_the_list_while_no_measure_is_kept() {
    let drawn = |app: App, window| app.handled(Message::Drawn(window)).1.units();
    let (app, main) = windowed();
    assert_eq!(drawn(app, main), 1);
    let (app, _) = windowed();
    assert_eq!(drawn(app, window::Id::unique()), 0);
    let (app, main) = windowed();
    assert_eq!(drawn(measured_list(app), main), 0);
}

#[test]
fn three_pages_pressed_in_a_row_after_a_resize_each_move_a_full_step() {
    let (app, main) = windowed();
    let (resized, _) = app.handled(Message::Resized(main, Size::new(800.0, 600.0)));
    let first = typed(measured_list(resized), "f", &[]).0;
    assert_eq!(first.selected, Some(CueId(4)));
    let second = typed(first, "f", &[]).0;
    assert_eq!(second.selected, Some(CueId(7)));
    let third = typed(second, "f", &[]).0;
    assert_eq!(third.selected, Some(CueId(12)));
}
