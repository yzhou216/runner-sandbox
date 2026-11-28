// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the labels that hint mode gives out

use std::collections::HashSet;

use super::{LETTERS, labels};

#[test]
fn up_to_one_label_per_letter_take_one_letter_each_in_the_order_of_the_letters() {
    assert!(labels(0).is_empty());
    assert_eq!(labels(1), ["s"]);
    assert_eq!(labels(3), ["s", "a", "d"]);
    assert_eq!(labels(LETTERS.len()).concat(), LETTERS);
}

#[test]
fn one_more_label_than_letters_takes_two_letters_for_every_label() {
    let many = labels(LETTERS.len() + 1);
    assert!(many.iter().all(|label| label.len() == 2));
    assert_eq!(many[..3], ["ss", "sa", "sd"]);
    assert_eq!(many[LETTERS.len()], "as");
}

#[test]
fn no_label_begins_another_and_none_repeats() {
    [1, 5, 14, 15, 60, 196, 197, 500]
        .into_iter()
        .for_each(|count| {
            let all = labels(count);
            assert_eq!(all.len(), count);
            assert_eq!(all.iter().collect::<HashSet<_>>().len(), count, "{count}");
            assert!(
                all.iter().all(|one| all
                    .iter()
                    .all(|other| one == other || !other.starts_with(one))),
                "{count}"
            );
        });
}

#[test]
fn three_letters_come_only_after_two_run_out() {
    assert_eq!(labels(196).first().map(String::len), Some(2));
    assert_eq!(labels(197).first().map(String::len), Some(3));
}
