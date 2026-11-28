// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which places the walk finds in sight, and of where their labels lie

use iced::{
    Point, Rectangle, Size, Vector,
    advanced::widget::{Operation, operation::Outcome},
};

use super::{
    Act, Hidden, Site, Target, Window,
    layer::{PADDING, corners, width},
    visit::walk,
};
use crate::front::{
    Message,
    fixtures::Scrolled,
    view::{char_width, line_height},
};

fn rectangle(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
    Rectangle::new(Point::new(x, y), Size::new(width, height))
}

/// Tells the operation of a place whose act is the switch to list `number`
fn place(operation: &mut dyn Operation<Vec<Site>>, number: usize, bounds: Rectangle) {
    operation.custom(
        None,
        bounds,
        &mut Target(Act::Send(Message::SwitchList(number))),
    );
}

/// Walks as `walking` leads, and returns the numbers of the places in sight
fn seen(walking: impl FnOnce(&mut dyn Operation<Vec<Site>>)) -> Vec<usize> {
    let mut operation = walk();
    walking(&mut operation);
    let Outcome::Some(places) = operation.finish() else {
        panic!("the walk returns what it found");
    };
    places
        .iter()
        .map(|place| match place.act {
            Act::Send(Message::SwitchList(number)) => number,
            ref other => panic!("{other:?}"),
        })
        .collect()
}

#[test]
fn places_in_the_window_are_found_in_the_order_of_the_walk() {
    let found = seen(|operation| {
        (0..3).for_each(|row| place(operation, row, rectangle(0.0, 10.0 * row as f32, 5.0, 5.0)));
    });
    assert_eq!(found, [0, 1, 2]);
}

#[test]
fn a_place_scrolled_out_of_the_scrollable_is_left_out() {
    // Scrollable shows 0 to 100 of a content 300 high, scrolled 150 down
    let found = seen(|operation| {
        operation.scrollable(
            None,
            rectangle(0.0, 0.0, 100.0, 100.0),
            rectangle(0.0, 0.0, 100.0, 300.0),
            Vector::new(0.0, 150.0),
            &mut Scrolled::default(),
        );
        operation.traverse(&mut |operation| {
            // Places lie at 40, 160, 260 in the content, so that they show
            // at -110, 10, 110
            [40.0, 160.0, 260.0]
                .into_iter()
                .enumerate()
                .for_each(|(number, y)| place(operation, number, rectangle(0.0, y, 50.0, 20.0)));
        });
        // Site after the scrollable lies in the window again
        place(operation, 3, rectangle(0.0, 300.0, 50.0, 20.0));
    });
    assert_eq!(found, [1, 3]);
}

#[test]
fn a_place_in_a_scrollable_that_lies_in_another_is_judged_by_both() {
    // Inner scrollable shows 50 to 150 of the window, in an outer one that
    // shows 0 to 100 and has scrolled 20, so that only 50 to 100 of the inner
    // shows, which is the layout's 70 to 120 less the outer's scroll
    let found = seen(|operation| {
        operation.scrollable(
            None,
            rectangle(0.0, 0.0, 100.0, 100.0),
            rectangle(0.0, 0.0, 100.0, 400.0),
            Vector::new(0.0, 20.0),
            &mut Scrolled::default(),
        );
        operation.traverse(&mut |operation| {
            operation.scrollable(
                None,
                rectangle(0.0, 70.0, 100.0, 100.0),
                rectangle(0.0, 70.0, 100.0, 300.0),
                Vector::ZERO,
                &mut Scrolled::default(),
            );
            operation.traverse(&mut |operation| {
                // Show at 60, 110, and 40 in the window
                [(80.0, 0), (130.0, 1), (60.0, 2)]
                    .into_iter()
                    .for_each(|(y, number)| {
                        place(operation, number, rectangle(0.0, y, 50.0, 20.0))
                    });
            });
        });
    });
    assert_eq!(found, [0]);
}

#[test]
fn a_place_outside_the_window_is_left_out() {
    let found = seen(|operation| {
        operation.custom(None, rectangle(0.0, 0.0, 200.0, 100.0), &mut Window);
        place(operation, 0, rectangle(150.0, 50.0, 40.0, 20.0));
        place(operation, 1, rectangle(250.0, 50.0, 40.0, 20.0));
    });
    assert_eq!(found, [0]);
}

#[test]
fn a_place_in_hidden_content_is_left_out_and_content_after_it_is_not() {
    let found = seen(|operation| {
        operation.custom(None, Rectangle::default(), &mut Hidden(true));
        place(operation, 0, rectangle(0.0, 0.0, 5.0, 5.0));
        operation.custom(None, Rectangle::default(), &mut Hidden(false));
        place(operation, 1, rectangle(0.0, 0.0, 5.0, 5.0));
    });
    assert_eq!(found, [1]);
}

#[test]
fn a_place_keeps_its_layout_bounds_and_the_bounds_that_it_shows_at() {
    let mut operation = walk();
    operation.scrollable(
        None,
        rectangle(0.0, 0.0, 100.0, 100.0),
        rectangle(0.0, 0.0, 100.0, 300.0),
        Vector::new(0.0, 150.0),
        &mut Scrolled::default(),
    );
    operation.traverse(&mut |operation| place(operation, 0, rectangle(5.0, 160.0, 50.0, 20.0)));
    let Outcome::Some(places) = operation.finish() else {
        panic!("the walk returns what it found");
    };
    assert_eq!(places[0].at, rectangle(5.0, 160.0, 50.0, 20.0));
    assert_eq!(places[0].shown, rectangle(5.0, 10.0, 50.0, 20.0));
}

/// Returns the point that the label of a place that stands alone begins at
fn corner(place: Rectangle, letters: usize) -> Point {
    corners(&[(place, letters)])[0]
}

#[test]
fn a_label_lies_left_of_its_place_with_its_right_edge_at_the_left_edge_of_the_place() {
    let place = rectangle(300.0, 50.0, 40.0, 20.0);
    let at = corner(place, 2);
    let width = 2.0 * char_width() + 2.0 * PADDING;
    assert_eq!(at.y, 50.0);
    assert!((at.x + width - 300.0).abs() < 1e-3);
}

#[test]
fn a_label_of_more_letters_reaches_farther_left_from_the_same_edge() {
    let place = rectangle(300.0, 50.0, 40.0, 20.0);
    let widths = [1, 2, 3].map(|letters| 300.0 - corner(place, letters).x);
    assert!(widths.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn a_label_that_would_leave_the_window_lies_right_of_its_place() {
    assert_eq!(
        corner(rectangle(3.0, 50.0, 40.0, 20.0), 2),
        Point::new(43.0, 50.0)
    );
    assert_eq!(
        corner(rectangle(0.0, 5.0, 40.0, 20.0), 1),
        Point::new(40.0, 5.0)
    );
}

#[test]
fn a_label_right_of_its_place_gives_way_to_a_label_there_by_lying_at_the_window_edge() {
    // A wide button at the window edge, and a check box 11 pixels right of it
    let button = rectangle(10.0, 100.0, 174.0, 30.0);
    let check_box = rectangle(195.0, 104.0, 15.0, 15.0);
    let at = corners(&[(button, 1), (check_box, 1)]);
    assert_eq!(at[0], Point::new(0.0, 100.0));
    assert_eq!(at[1], corner(check_box, 1));
    let boxes: Vec<_> = at
        .iter()
        .map(|at| rectangle(at.x, at.y, width(1), line_height()))
        .collect();
    assert!(!boxes[0].intersects(&boxes[1]));
}

#[test]
fn a_label_right_of_its_place_stays_there_where_no_label_lies_there() {
    let button = rectangle(10.0, 100.0, 174.0, 30.0);
    let check_box = rectangle(435.0, 104.0, 15.0, 15.0);
    assert_eq!(
        corners(&[(button, 1), (check_box, 1)])[0],
        Point::new(184.0, 100.0)
    );
}

/// Returns whether any two of the labels at `at`, each of `letters` letters,
/// lie on each other
fn overlap(at: &[Point], letters: &[usize]) -> bool {
    let boxes: Vec<_> = at
        .iter()
        .zip(letters)
        .map(|(at, &letters)| rectangle(at.x, at.y, width(letters), line_height()))
        .collect();
    boxes
        .iter()
        .enumerate()
        .any(|(index, one)| boxes[index + 1..].iter().any(|other| one.intersects(other)))
}

#[test]
fn labels_that_both_fall_back_to_the_window_edge_on_one_row_lie_apart() {
    // Two places near the window edge, whose right sides a long label that
    // lies left of its place on the same row covers, so both fall back
    let letters = (300.0 / char_width()).ceil() as usize;
    let wall = rectangle(400.0, 100.0, 40.0, 20.0);
    let labels = [
        (rectangle(2.0, 100.0, 100.0, 20.0), 1),
        (rectangle(3.0, 100.0, 100.0, 20.0), 1),
        (wall, letters),
    ];
    let at = corners(&labels);
    assert!(!overlap(&at, &[1, 1, letters]), "{at:?}");
}

#[test]
fn a_label_at_the_window_edge_leaves_a_label_left_of_its_place_in_view() {
    // The first label lies left of its place and starts 5 pixels from the
    // window edge, and the second, which has more letters, cannot lie right
    // of its place, so it falls back to the edge
    let near = rectangle(5.0 + width(1), 100.0, 10.0, 20.0);
    let crowded = rectangle(3.0, 100.0, 10.0, 20.0);
    let at = corners(&[(near, 1), (crowded, 3)]);
    assert!(!overlap(&at, &[1, 3]), "{at:?}");
}
