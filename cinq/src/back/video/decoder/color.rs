// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures that the decoder hands out, turned into RGBA with the colors of
//! the standard that their size implies
//!
//! High-definition video takes the matrix of BT.709, and smaller video that
//! of BT.601, both in their limited range.

#[cfg(test)]
mod tests;

use rust_h264::decoder::Frame;
use yuv::{YuvPlanarImage, YuvRange, YuvStandardMatrix};

/// Width and lines from either of which a picture counts as high
/// definition, so that a wide picture cropped from one of 1280 by 720, such
/// as one of 1280 by 534, does as well
const HIGH_DEFINITION: (u32, u32) = (1280, 720);

/// Picture in RGBA, a byte for each channel, row by row from the top
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Returns a decoded picture in RGBA, which holds its colors in eight bits
/// and shares them among four pixels
pub(super) fn rgba(picture: &Frame) -> Picture {
    let (width, height) = (picture.width, picture.height);
    let matrix = if high_definition(picture) {
        YuvStandardMatrix::Bt709
    } else {
        YuvStandardMatrix::Bt601
    };
    // yuv converts whole squares of four pixels, so a picture of an odd
    // size, which only monochrome video has, leaves its last row or column
    // out, transparent
    let planes = YuvPlanarImage {
        y_plane: &picture.y,
        y_stride: width,
        u_plane: &picture.u,
        u_stride: picture.chroma_width(),
        v_plane: &picture.v,
        v_stride: picture.chroma_width(),
        width: width & !1,
        height: height & !1,
    };
    let mut pixels = vec![0; width as usize * height as usize * 4];
    // Picture that a crafted stream crops to under two pixels has nothing to
    // convert.  The decoder sizes the planes to the picture, which leaves
    // the conversion no other way to fail.
    if planes.width > 0 && planes.height > 0 {
        let _ = yuv::yuv420_to_rgba(&planes, &mut pixels, width * 4, YuvRange::Limited, matrix);
    }
    Picture {
        width,
        height,
        pixels,
    }
}

fn high_definition(picture: &Frame) -> bool {
    let (width, lines) = HIGH_DEFINITION;
    picture.width >= width || picture.height >= lines
}
