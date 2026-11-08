// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the colors that pictures take with BT.601 and BT.709, by their
//! size

use rust_h264::decoder::{ChromaFormat, Frame};

use super::{BT601, BT709, HIGH_DEFINITION, Matrix, converted, rgba};

/// Returns a picture `width` pixels wide of `lines` lines of `luma`, which
/// share the differences of color `blue` and `red`
fn picture(width: u32, lines: u32, luma: u8, blue: u8, red: u8) -> Frame {
    let pixels = (width * lines) as usize;
    Frame {
        width,
        height: lines,
        y: vec![luma; pixels],
        u: vec![blue; pixels / 4],
        v: vec![red; pixels / 4],
        pic_order_cnt: 0,
        chroma_format: ChromaFormat::Yuv420,
        bit_depth: 8,
    }
}

/// Returns the RGBA of a picture of four pixels of `luma`, which share the
/// differences of color `blue` and `red`, with `matrix`
fn converted_with(matrix: Matrix, luma: u8, blue: u8, red: u8) -> Vec<u8> {
    converted(&picture(2, 2, luma, blue, red), matrix)
}

/// Returns whether `pixels` are all of the RGBA of `color`, but for one step
/// of rounding
fn all_close(pixels: &[u8], color: [u8; 4]) -> bool {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .all(|pixel| pixel.iter().zip(color).all(|(&a, b)| a.abs_diff(b) <= 1))
}

#[test]
fn black_gray_and_white_keep_their_shade() {
    [BT601, BT709].into_iter().for_each(|matrix| {
        assert!(all_close(
            &converted_with(matrix, 16, 128, 128),
            [0, 0, 0, 255]
        ));
        assert!(all_close(
            &converted_with(matrix, 126, 128, 128),
            [128, 128, 128, 255]
        ));
        assert!(all_close(
            &converted_with(matrix, 235, 128, 128),
            [255, 255, 255, 255]
        ));
    });
}

#[test]
fn red_of_each_standard_comes_back_red() {
    // Pure red as BT.601 holds it, and as BT.709 does, which BT.601 would
    // show as a duller red with a little blue, (233, 0, 2)
    assert!(all_close(
        &converted_with(BT601, 81, 90, 240),
        [255, 0, 0, 255]
    ));
    assert!(all_close(
        &converted_with(BT709, 63, 102, 240),
        [255, 0, 0, 255]
    ));
}

#[test]
fn pictures_take_the_matrix_of_their_size() {
    let (_, lines) = HIGH_DEFINITION;
    let smaller = rgba(&picture(2, lines - 2, 81, 90, 240));
    assert!(all_close(&smaller.pixels, [255, 0, 0, 255]));
    let high = rgba(&picture(2, lines, 63, 102, 240));
    assert!(all_close(&high.pixels, [255, 0, 0, 255]));
    assert_eq!((high.width, high.height), (2, lines));
}

#[test]
fn wide_picture_cropped_from_high_definition_takes_its_matrix() {
    let (width, _) = HIGH_DEFINITION;
    let narrower = rgba(&picture(width - 2, 534, 81, 90, 240));
    assert!(all_close(&narrower.pixels, [255, 0, 0, 255]));
    let scope = rgba(&picture(width, 534, 63, 102, 240));
    assert!(all_close(&scope.pixels, [255, 0, 0, 255]));
}

#[test]
fn every_row_of_a_high_definition_picture_converts_whatever_its_strips() {
    // Strips of 91 rows on eight cores, so that some start on odd rows
    let (_, lines) = HIGH_DEFINITION;
    let high = rgba(&picture(2, lines + 2, 63, 102, 240));
    assert!(all_close(&high.pixels, [255, 0, 0, 255]));
}

#[test]
fn picture_cropped_to_no_width_converts_to_no_pixels() {
    let (_, lines) = HIGH_DEFINITION;
    assert!(rgba(&picture(0, lines, 63, 102, 240)).pixels.is_empty());
    assert!(rgba(&picture(0, 2, 63, 102, 240)).pixels.is_empty());
}
