// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::geo::Crs;

const GT_MODEL_TYPE: u16 = 1024;
const GT_RASTER_TYPE: u16 = 1025;
const GEOGRAPHIC_TYPE: u16 = 2048;
const PROJECTED_CS_TYPE: u16 = 3072;
const MODEL_TYPE_PROJECTED: u16 = 1;
const MODEL_TYPE_GEOGRAPHIC: u16 = 2;
const RASTER_PIXEL_IS_AREA: u16 = 1;
const RASTER_PIXEL_IS_POINT: u16 = 2;
const UNDEFINED: u16 = 0;
const USER_DEFINED: u16 = 32767;

/// CRS declared by a GeoKeyDirectory, or `None` if it has no EPSG code.
///
/// The model type tells a projected CRS from the geographic CRS it is based
/// on, which the directory may also name.
pub fn decode(keys: &[u16]) -> Option<Crs> {
    let projected = || {
        inline_value(keys, PROJECTED_CS_TYPE)
            .and_then(epsg)
            .map(Crs::Projected)
    };
    let geographic = || {
        inline_value(keys, GEOGRAPHIC_TYPE)
            .and_then(epsg)
            .map(Crs::Geographic)
    };
    match inline_value(keys, GT_MODEL_TYPE) {
        Some(MODEL_TYPE_PROJECTED) => projected(),
        Some(MODEL_TYPE_GEOGRAPHIC) => geographic(),
        Some(_) => None,
        // Without a model type, a projected CRS key implies a projected model
        None if inline_value(keys, PROJECTED_CS_TYPE).is_some() => projected(),
        None => geographic(),
    }
}

/// `code` unless it marks a CRS as undefined or defined by other keys.
fn epsg(code: u16) -> Option<u16> {
    (code != UNDEFINED && code != USER_DEFINED).then_some(code)
}

/// Whether a GeoKeyDirectory declares pixels to represent points, so that
/// tie points refer to pixel centers rather than corners.
pub fn pixel_is_point(keys: &[u16]) -> bool {
    inline_value(keys, GT_RASTER_TYPE) == Some(RASTER_PIXEL_IS_POINT)
}

/// GeoKeyDirectory declaring `crs`, with pixels representing areas.
pub fn encode(crs: Crs) -> [[u16; 4]; 4] {
    let (model_type, crs_key) = match crs {
        Crs::Geographic(_) => (MODEL_TYPE_GEOGRAPHIC, GEOGRAPHIC_TYPE),
        Crs::Projected(_) => (MODEL_TYPE_PROJECTED, PROJECTED_CS_TYPE),
    };
    [
        // Version 1.1.0 header followed by three keys
        [1, 1, 0, 3],
        [GT_MODEL_TYPE, 0, 1, model_type],
        [GT_RASTER_TYPE, 0, 1, RASTER_PIXEL_IS_AREA],
        [crs_key, 0, 1, crs.epsg()],
    ]
}

/// Value of `key` when stored in the directory itself rather than in another
/// tag.  Only as many entries as the header counts are keys, since values that
/// the keys refer to may follow them.
fn inline_value(keys: &[u16], key: u16) -> Option<u16> {
    let count = usize::from(*keys.get(3)?);
    keys.get(4..)?
        .as_chunks::<4>()
        .0
        .iter()
        .take(count)
        .find(|entry| entry[0] == key && entry[1] == 0)
        .map(|entry| entry[3])
}
