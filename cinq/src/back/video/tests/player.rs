// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the pictures that a player hands on as the clock of its run
//! moves, through the passes of a clip that loops as well

use std::{thread, time::Duration};

use super::{
    clip,
    movie::{movie, picture_number},
    patiently,
};
use crate::back::{Shown, video::player::Player};

/// Moves `player` on to `played` seconds of its clip until it hands on the
/// picture numbered `expected`, and returns the player with the numbers of
/// every picture that it handed on
pub(super) fn shown(player: Player, played: f64, expected: usize) -> (Player, Vec<usize>) {
    patiently((player, Vec::new()), |(player, seen)| {
        let (player, picture) = player.advanced(played).expect("the pictures decode");
        let seen = [
            seen,
            picture.map_or_else(Vec::new, |picture| vec![picture_number(&picture.pixels)]),
        ]
        .concat();
        let done = seen.last() == Some(&expected);
        ((player, seen), done)
    })
}

/// Moves `player` on to the end of its clip in the time that a decoding
/// thread would need to hand on another picture, and returns how many
/// pictures it handed on, which are none once its last pass has ended
pub(super) fn pictures_after_the_end(player: Player) -> usize {
    (0..40)
        .fold((player, 0), |(player, pictures), _| {
            thread::sleep(Duration::from_millis(5));
            let (player, picture) = player.advanced(100.0).expect("the pictures decode");
            (player, pictures + usize::from(picture.is_some()))
        })
        .1
}

#[test]
fn player_hands_on_the_newest_picture_that_its_clock_has_reached() {
    let (_directory, path) = movie(15, 5);
    let player = Player::start(path, Shown::Clip(clip(0.0, 0.5, false)));
    let (player, first) = shown(player, 0.0, 0);
    assert_eq!(first, [0]);
    // Picture 6 shows at a fifth of a second, and picture 7 just after
    let (player, later) = shown(player, 0.2, 6);
    assert!(
        later.iter().all(|&number| (1..=6).contains(&number)),
        "{later:?}"
    );
    assert_eq!(later.last(), Some(&6));
    let (_, held) = player.advanced(0.2).expect("the pictures decode");
    assert!(held.is_none());
}

#[test]
fn player_of_a_clip_that_loops_shows_its_pictures_again_on_each_pass() {
    let (_directory, path) = movie(12, 4);
    // Pictures 3 to 5 make a tenth of a second, which loops after picture 5
    let player = Player::start(path, Shown::Clip(clip(0.1, 0.2, true)));
    let (player, first) = shown(player, 0.09, 5);
    assert_eq!(first.last(), Some(&5));
    let (player, second) = shown(player, 0.11, 3);
    assert_eq!(second, [3]);
    // Pictures 4 and 5 of the second pass are due as well by the third,
    // and either may come first
    let (_, third) = shown(player, 0.21, 3);
    let (last, before) = third.split_last().expect("pictures come");
    assert!(
        *last == 3 && before.iter().all(|&number| number > 3),
        "{third:?}"
    );
}
