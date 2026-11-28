// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the order in which a file and a stream declare their colors

use mp4_atom::Colr;
use rust_h264::nal::SpsInfo;
use yuv::YuvStandardMatrix;

use super::resolved;

fn sps(matrix: u8, full_range: bool) -> SpsInfo {
    SpsInfo {
        width: 16,
        height: 16,
        coded_width: 16,
        coded_height: 16,
        profile: 100,
        chroma_format: 1,
        bit_depth_luma: 8,
        bit_depth_chroma: 8,
        reference_frames: 1,
        progressive: true,
        full_range,
        primaries: 2,
        transfer: 2,
        matrix,
        chroma_location: 0,
        max_reorder_frames: None,
    }
}

fn nclx(matrix: u16, full_range: bool) -> Colr {
    Colr::new(1, 1, matrix, full_range).unwrap()
}

#[test]
fn a_file_and_a_set_declare_their_colors_in_order() {
    use YuvStandardMatrix::{Bt601, Bt709};
    // The file as (matrix, full range) of its nclx box, the set likewise, and
    // the matrix and range that come out
    [
        (None, None, None, false),
        (None, Some((2, false)), None, false),
        (None, Some((1, true)), Some(Bt709), true),
        (None, Some((5, false)), Some(Bt601), false),
        (None, Some((6, false)), Some(Bt601), false),
        (Some((6, false)), Some((1, true)), Some(Bt601), false),
        (Some((1, true)), Some((6, false)), Some(Bt709), true),
        (Some((2, true)), Some((1, false)), Some(Bt709), true),
    ]
    .into_iter()
    .enumerate()
    .for_each(|(row, (file, set, matrix, full_range))| {
        let file: Vec<_> = file
            .map(|(matrix, full)| nclx(matrix, full))
            .into_iter()
            .collect();
        let declared = resolved(&file, set.map(|(matrix, full)| sps(matrix, full)));
        assert_eq!(
            (declared.matrix, declared.full_range),
            (matrix, full_range),
            "row {row}"
        );
    });
}

#[test]
fn a_color_box_of_a_profile_declares_nothing() {
    let profile = Colr::Prof { profile: vec![0] };
    let declared = resolved(&[profile], Some(sps(1, true)));
    assert_eq!(
        (declared.matrix, declared.full_range),
        (Some(YuvStandardMatrix::Bt709), true)
    );
}
