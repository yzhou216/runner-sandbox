// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the colors that pictures take with BT.601 and BT.709, by their
//! size

use rust_h264::decoder::{ChromaFormat, Frame};

use super::{HIGH_DEFINITION, rgba};

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

fn all_close(pixels: &[u8], color: [u8; 4]) -> bool {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .all(|pixel| pixel.iter().zip(color).all(|(&a, b)| a.abs_diff(b) <= 1))
}

#[test]
fn black_gray_and_white_keep_their_shade() {
    let (_, lines) = HIGH_DEFINITION;
    [2, lines].into_iter().for_each(|lines| {
        assert!(all_close(
            &rgba(&picture(2, lines, 16, 128, 128)).pixels,
            [0, 0, 0, 255]
        ));
        assert!(all_close(
            &rgba(&picture(2, lines, 126, 128, 128)).pixels,
            [128, 128, 128, 255]
        ));
        assert!(all_close(
            &rgba(&picture(2, lines, 235, 128, 128)).pixels,
            [255, 255, 255, 255]
        ));
    });
}

#[test]
fn pictures_take_the_matrix_of_their_size() {
    let (_, lines) = HIGH_DEFINITION;
    let smaller = rgba(&picture(2, lines - 2, 81, 90, 240));
    assert!(all_close(&smaller.pixels, [255, 0, 0, 255]));
    let high = rgba(&picture(2, lines, 63, 102, 240));
    assert!(all_close(&high.pixels, [255, 0, 0, 255]));
}

#[test]
fn picture_cropped_to_no_width_converts_to_no_pixels() {
    let (_, lines) = HIGH_DEFINITION;
    assert!(rgba(&picture(0, lines, 63, 102, 240)).pixels.is_empty());
    assert!(rgba(&picture(0, 2, 63, 102, 240)).pixels.is_empty());
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
fn each_pixel_of_a_square_takes_the_shade_of_its_own_luma() {
    let square = Frame {
        y: vec![16, 235, 235, 16],
        ..picture(2, 2, 0, 128, 128)
    };
    let pixels = rgba(&square).pixels;
    let (black, white) = ([0, 0, 0, 255], [255, 255, 255, 255]);
    [black, white, white, black]
        .iter()
        .zip(pixels.as_chunks::<4>().0)
        .for_each(|(&color, pixel)| assert!(all_close(pixel, color), "{pixel:?}"));
}

#[test]
fn picture_of_an_odd_size_converts_all_but_its_last_row_and_column() {
    let pixels = rgba(&picture(3, 3, 126, 128, 128)).pixels;
    let rows: Vec<_> = pixels.as_chunks::<4>().0.chunks(3).collect();
    rows[..2].iter().for_each(|row| {
        assert!(all_close(row[..2].as_flattened(), [128, 128, 128, 255]));
        assert_eq!(row[2], [0, 0, 0, 0]);
    });
    assert!(rows[2].iter().all(|&pixel| pixel == [0, 0, 0, 0]));
}
