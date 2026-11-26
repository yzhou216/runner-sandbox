// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt::Display, str::FromStr};

use num_traits::FromPrimitive;
use tiff::{
    decoder::DecodingResult,
    encoder::colortype::{ColorType, Gray8, Gray32Float},
};

/// Sample type that can be stored in a single-band GeoTIFF.
pub trait Sample: Copy + PartialEq + FromStr + Display + FromPrimitive + Into<f64> {
    type Color: ColorType<Inner = Self>;

    /// Decoded samples as this type, if they convert to it without loss.
    fn from_decoded(result: DecodingResult) -> Option<Vec<Self>>;
}

impl Sample for u8 {
    type Color = Gray8;

    fn from_decoded(result: DecodingResult) -> Option<Vec<Self>> {
        match result {
            DecodingResult::U8(data) => Some(data),
            _ => None,
        }
    }
}

impl Sample for f32 {
    type Color = Gray32Float;

    fn from_decoded(result: DecodingResult) -> Option<Vec<Self>> {
        match result {
            DecodingResult::U8(data) => Some(data.into_iter().map(Self::from).collect()),
            DecodingResult::I8(data) => Some(data.into_iter().map(Self::from).collect()),
            DecodingResult::U16(data) => Some(data.into_iter().map(Self::from).collect()),
            DecodingResult::I16(data) => Some(data.into_iter().map(Self::from).collect()),
            DecodingResult::F32(data) => Some(data),
            _ => None,
        }
    }
}
