// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the times that the pictures of a pass take, from a decoder
//! that hands out the pictures of B-frames in the order they show

use super::Times;

/// What a decoder does next with the samples of a pass: hands out a
/// picture, or reaches a closed group of pictures that starts at a time
#[derive(Clone, Copy)]
enum Step {
    Picture,
    Group(f64),
}

/// Returns the times that the pictures take as a decoder takes `steps`
fn taken(shows: &[f64], steps: &[Step]) -> Vec<f64> {
    let start = (Times::new(shows.to_vec()), Vec::new());
    let (_, taken) = steps.iter().fold(start, |(times, taken), step| match step {
        Step::Picture => {
            let (time, times) = times.next();
            (times, [taken, time.into_iter().collect()].concat())
        }
        Step::Group(shows) => (times.from(*shows), taken),
    });
    taken
}

/// Times of two closed groups of I, P, and two B-frames, which decode in
/// that order: pictures 0 to 3, then 4 to 7
const SHOWS: [f64; 8] = [0.0, 3.0, 1.0, 2.0, 4.0, 7.0, 5.0, 6.0];

#[test]
fn pictures_take_the_times_they_show_in_order() {
    let steps = [
        [Step::Picture, Step::Picture, Step::Picture, Step::Picture],
        [Step::Picture, Step::Picture, Step::Picture, Step::Picture],
    ];
    let times = taken(&SHOWS, &steps.concat());
    let expected: Vec<f64> = (0..8).map(f64::from).collect();
    assert_eq!(times, expected);
}

#[test]
fn picture_that_fails_to_decode_shifts_no_time_past_its_own_group() {
    // Picture 1 of the first group fails, which leaves pictures 2 and 3 a
    // time early, until the second group starts the times again
    let steps = [
        vec![Step::Picture, Step::Picture, Step::Picture],
        vec![Step::Group(4.0)],
        vec![Step::Picture, Step::Picture, Step::Picture, Step::Picture],
    ];
    let times = taken(&SHOWS, &steps.concat());
    assert_eq!(times, [0.0, 1.0, 2.0, 4.0, 5.0, 6.0, 7.0]);
}

#[test]
fn pictures_past_the_last_time_take_none() {
    let (time, times) = Times::new(vec![1.0]).next();
    assert_eq!(time, Some(1.0));
    assert_eq!(times.next().0, None);
}
