// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the colors that high-definition pictures take with BT.709

use openh264::formats::YUVBuffer;

use super::bt709;

/// Returns the RGBA of a picture of four pixels of `luma`, which share the
/// differences of color `blue` and `red`, with BT.709
fn converted(luma: u8, blue: u8, red: u8) -> Vec<u8> {
    let planes = [vec![luma; 4], vec![blue], vec![red]].concat();
    bt709(&YUVBuffer::from_vec(planes, 2, 2))
}

/// Returns whether `pixels` are four of the RGBA of `color`, but for one
/// step of rounding
fn all_close(pixels: &[u8], color: [u8; 4]) -> bool {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .all(|pixel| pixel.iter().zip(color).all(|(&a, b)| a.abs_diff(b) <= 1))
}

#[test]
fn black_gray_and_white_keep_their_shade() {
    assert!(all_close(&converted(16, 128, 128), [0, 0, 0, 255]));
    assert!(all_close(&converted(126, 128, 128), [128, 128, 128, 255]));
    assert!(all_close(&converted(235, 128, 128), [255, 255, 255, 255]));
}

#[test]
fn red_of_bt709_comes_back_red() {
    // Pure red as BT.709 holds it, which BT.601 would show as a duller red
    // with a little blue, (233, 0, 2)
    assert!(all_close(&converted(63, 102, 240), [255, 0, 0, 255]));
}
