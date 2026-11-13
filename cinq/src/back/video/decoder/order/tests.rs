// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the order in which the pictures of B-frames go out, and of the
//! times they take when a picture fails to decode

use super::Order;

/// Times of two closed groups of I, P, and two B-frames, which decode in
/// that order: pictures 0 to 3, then 4 to 7
const SHOWS: [f64; 8] = [0.0, 3.0, 1.0, 2.0, 4.0, 7.0, 5.0, 6.0];

/// Returns the pictures that go out, each named by the time it shows and
/// with the time it takes, as the samples of `SHOWS` decode in turn, to
/// pictures but for those at the places of `failed`, and how many went out
/// as each sample decoded
fn sent(failed: &[usize]) -> (Vec<(f64, f64)>, Vec<usize>) {
    let start = (Order::new(SHOWS), Vec::new(), Vec::new());
    let (_, sent, counts) =
        SHOWS
            .iter()
            .enumerate()
            .fold(start, |(order, sent, counts), (place, &shows)| {
                let picture = (!failed.contains(&place)).then_some(shows);
                let (due, order) = order.decoded(place, picture);
                let count = due.len();
                (order, [sent, due].concat(), [counts, vec![count]].concat())
            });
    (sent, counts)
}

/// Returns pictures named by the times of `times`, each with its own time
fn named(times: &[u8]) -> Vec<(f64, f64)> {
    times
        .iter()
        .map(|&time| (f64::from(time), f64::from(time)))
        .collect()
}

#[test]
fn pictures_go_out_in_the_order_they_show_once_those_before_them_decode() {
    let (sent, counts) = sent(&[]);
    assert_eq!(sent, named(&[0, 1, 2, 3, 4, 5, 6, 7]));
    // P-frames wait for the B-frames that decode after them and show before
    assert_eq!(counts, [1, 0, 1, 2, 1, 0, 1, 2]);
}

#[test]
fn picture_that_fails_to_decode_costs_no_other_picture_its_time() {
    let (sent, _) = sent(&[2]);
    assert_eq!(sent, named(&[0, 2, 3, 4, 5, 6, 7]));
}
