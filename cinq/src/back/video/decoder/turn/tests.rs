// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the turns that a matrix makes, and of turning a picture by them

use mp4_atom::Matrix;

use super::{ONE, Picture, Turn};

fn matrix(a: i32, b: i32, c: i32, d: i32) -> Matrix {
    Matrix {
        a,
        b,
        u: 0,
        c,
        d,
        v: 0,
        x: 0,
        y: 0,
        w: 1 << 30,
    }
}

/// Returns a picture of three pixels across and two down, whose pixels
/// count from 1 as the rows read
fn counted() -> Picture {
    Picture {
        width: 3,
        height: 2,
        pixels: (1..=6).flat_map(|number| [number; 4]).collect(),
    }
}

/// Returns the numbers of the pixels of `picture` as its rows read
fn numbers(picture: &Picture) -> Vec<u8> {
    picture.pixels.chunks(4).map(|pixel| pixel[0]).collect()
}

#[test]
fn matrices_of_right_angles_turn_clockwise_and_the_rest_do_not() {
    let turns = [
        (matrix(ONE, 0, 0, ONE), Turn::None),
        (matrix(0, ONE, -ONE, 0), Turn::Quarter),
        (matrix(-ONE, 0, 0, -ONE), Turn::Half),
        (matrix(0, -ONE, ONE, 0), Turn::ThreeQuarters),
        (matrix(-ONE, 0, 0, ONE), Turn::None),
        (matrix(ONE / 2, 0, 0, ONE / 2), Turn::None),
    ];
    turns
        .iter()
        .for_each(|(matrix, turn)| assert_eq!(Turn::of(matrix), *turn));
}

#[test]
fn picture_turns_clockwise_by_each_turn() {
    let turned = |turn| {
        let picture = counted().turned(turn);
        (picture.width, picture.height, numbers(&picture))
    };
    assert_eq!(turned(Turn::None), (3, 2, vec![1, 2, 3, 4, 5, 6]));
    assert_eq!(turned(Turn::Quarter), (2, 3, vec![4, 1, 5, 2, 6, 3]));
    assert_eq!(turned(Turn::Half), (3, 2, vec![6, 5, 4, 3, 2, 1]));
    assert_eq!(turned(Turn::ThreeQuarters), (2, 3, vec![3, 6, 2, 5, 1, 4]));
}
