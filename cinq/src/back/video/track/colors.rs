// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Colors that a track declares: in the colr box of its description, else in
//! the VUI of its sequence parameter set

#[cfg(test)]
mod tests;

use mp4_atom::{Codec, Colr, Trak};
use rust_h264::nal::{NalUnit, NalUnitType, SpsInfo, parse_sps_info};
use yuv::YuvStandardMatrix;

use crate::back::video::decoder::Declared;

/// Returns what `trak`, whose parameter sets are `sets`, declares of its
/// colors.  The colr box comes first, as a player that reads the file does
/// before it reads the stream, and a matrix that it leaves unspecified is
/// the one of the set.
pub(super) fn declared(trak: &Trak, sets: &[NalUnit<'_>]) -> Declared {
    let colr = trak
        .mdia
        .minf
        .stbl
        .stsd
        .codecs
        .iter()
        .find_map(|codec| match codec {
            Codec::Avc1(avc) => Some(&avc.colr),
            Codec::Avc3(avc) => Some(&avc.colr),
            _ => None,
        });
    let sps = sets
        .iter()
        .filter(|unit| unit.nal_unit_type == NalUnitType::Sps)
        .find_map(|unit| parse_sps_info(unit).ok());
    resolved(colr.map_or(&[], Vec::as_slice), sps)
}

fn resolved(colr: &[Colr], sps: Option<SpsInfo>) -> Declared {
    let file = colr.iter().find_map(|colr| match *colr {
        Colr::Nclx {
            matrix_coefficients,
            full_range_flag,
            ..
        } => Some((matrix_coefficients, full_range_flag)),
        _ => None,
    });
    Declared {
        matrix: file
            .and_then(|(matrix, _)| standard(matrix))
            .or_else(|| sps.and_then(|sps| standard(sps.matrix.into()))),
        full_range: file.map_or_else(|| sps.is_some_and(|sps| sps.full_range), |(_, full)| full),
    }
}

/// Returns the matrix that a code of matrix coefficients names, none for the
/// codes that name no matrix of YCbCr, or that leave it unspecified
fn standard(code: u16) -> Option<YuvStandardMatrix> {
    match code {
        1 => Some(YuvStandardMatrix::Bt709),
        4 => Some(YuvStandardMatrix::Fcc),
        5 | 6 => Some(YuvStandardMatrix::Bt601),
        7 => Some(YuvStandardMatrix::Smpte240),
        9 => Some(YuvStandardMatrix::Bt2020),
        _ => None,
    }
}
