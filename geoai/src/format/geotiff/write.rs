// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    io::{BufWriter, Seek, Write},
    mem,
};

use anyhow::Result;
use tiff::{
    encoder::{Compression, TiffEncoder, TiffKind, TiffValue},
    tags::Tag,
};

use super::{Sample, geokeys};
use crate::raster::Raster;

const GDAL_METADATA: Tag = Tag::Unknown(42112);

/// Most bytes of samples written as a classic TIFF, which can hold 4 GiB,
/// leaving room for compression that enlarges data it cannot compress.
const CLASSIC_TIFF_BYTES: usize = 2 << 30;

#[derive(Clone, Default)]
pub struct Writer {
    compression: Compression,
    description: Option<String>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compression(self, compression: Compression) -> Self {
        Self {
            compression,
            ..self
        }
    }

    /// Band description, shown by GDAL and QGIS.
    pub fn description(self, description: impl Into<String>) -> Self {
        Self {
            description: Some(description.into()),
            ..self
        }
    }

    /// Write `raster` to `output`, organized in strips rather than tiles, and
    /// as a BigTIFF if it might not fit in a classic TIFF.
    pub fn write<T: Sample>(&self, raster: &Raster<T>, output: impl Write + Seek) -> Result<()>
    where
        [T]: TiffValue,
    {
        let mut file = BufWriter::new(output);
        if mem::size_of_val(raster.data()) > CLASSIC_TIFF_BYTES {
            self.encode(TiffEncoder::new_big(&mut file)?, raster)?;
        } else {
            self.encode(TiffEncoder::new(&mut file)?, raster)?;
        }
        // Dropping the buffer would flush it too, but would ignore a failure
        file.flush()?;
        Ok(())
    }

    fn encode<T: Sample, W: Write + Seek, K: TiffKind>(
        &self,
        encoder: TiffEncoder<W, K>,
        raster: &Raster<T>,
    ) -> Result<()>
    where
        [T]: TiffValue,
    {
        let grid = raster.grid();
        let ((west, north), (pixel_width, pixel_height)) = (grid.origin(), grid.pixel_size());

        let mut encoder = encoder.with_compression(self.compression);
        let mut image = encoder.new_image::<T::Color>(grid.width(), grid.height())?;
        let tags = image.encoder();
        tags.write_tag(
            Tag::ModelPixelScaleTag,
            &[pixel_width, pixel_height, 0.0][..],
        )?;
        tags.write_tag(
            Tag::ModelTiepointTag,
            &[0.0, 0.0, 0.0, west, north, 0.0][..],
        )?;
        if let Some(crs) = grid.crs() {
            tags.write_tag(Tag::GeoKeyDirectoryTag, geokeys::encode(crs).as_flattened())?;
        }
        if let Some(nodata) = raster.nodata() {
            tags.write_tag(Tag::GdalNodata, &*nodata.to_string())?;
        }
        if let Some(description) = &self.description {
            tags.write_tag(GDAL_METADATA, &*gdal_metadata(description))?;
        }
        image.write_data(raster.data())?;
        Ok(())
    }
}

fn gdal_metadata(description: &str) -> String {
    let escaped: String = description
        .chars()
        .map(|character| match character {
            '&' => "&amp;".to_owned(),
            '<' => "&lt;".to_owned(),
            '>' => "&gt;".to_owned(),
            // TIFF text is ASCII, so XML references stand for other characters
            _ if !character.is_ascii() => format!("&#{};", u32::from(character)),
            _ => character.to_string(),
        })
        .collect();
    format!(
        "<GDALMetadata>\n  <Item name=\"DESCRIPTION\" sample=\"0\" role=\"description\">{escaped}</Item>\n</GDALMetadata>\n"
    )
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek};

    use anyhow::Result;
    use tiff::encoder::TiffEncoder;

    use super::Writer;
    use crate::{format::geotiff, geo::Grid, raster::Raster};

    /// A raster written as a BigTIFF, as large rasters are, reads back.
    #[test]
    fn big_tiffs_read_back() -> Result<()> {
        let raster = Raster::new(Grid::new(2, 1), vec![1.5_f32, 2.5])?;
        let mut file = Cursor::new(Vec::new());
        Writer::new().encode(TiffEncoder::new_big(&mut file)?, &raster)?;
        file.rewind()?;
        assert_eq!(geotiff::read::<f32>(file)?.data(), raster.data());
        Ok(())
    }
}
