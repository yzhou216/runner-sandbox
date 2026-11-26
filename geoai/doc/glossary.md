<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# Glossary

Terms of geoai and of geospatial work, as the code and documentation use
them, in alphabetical order.

- **Band**: one grid of values within a raster file.  A satellite image may
  hold one band per color, and a climate dataset one band per date.  geoai
  reads single-band files.
- **Body**: the forms of a hazard definition after its declarations, which
  become a procedure of the inputs.  It runs once each time the hazard is
  computed and returns the raster of the hazard.
- **Catalog**: the hazards geoai knows by name, loaded from its built-in
  definitions, the user directory, and the paths given with `--hazards`.
- **Cloud Optimized GeoTIFF** (COG): a GeoTIFF laid out in tiles, with
  copies at lower resolutions, so that a program reading it over a network
  fetches only the parts it needs.
- **CRS** (coordinate reference system): the definition of what coordinates
  mean on the Earth, such as longitude and latitude in degrees, or meters
  east and north in a map projection.  geoai identifies a CRS by its EPSG
  code.
- **Declaration**: a form such as `(type u8)` after the signature of a
  hazard, describing the raster the hazard produces.
- **EPSG code**: the number of a CRS in the registry of the EPSG, such as 4326
  for longitude and latitude on WGS 84.
- **GeoTIFF**: a TIFF image with tags that georeference it.  geoai reads its
  inputs from GeoTIFF files and writes its outputs as GeoTIFF files.
- **Georeferencing**: the information that places a raster on the Earth: its
  CRS, the position of its corner, and the size of its pixels.
- **Grid**: the layout of the pixels of a raster: their number across and
  down, the position of the upper-left corner, the size of a pixel, and the
  CRS.
- **Hazard**: a named computation of a raster from inputs, such as earthquake
  hazard classes from peak ground acceleration.
- **Input**: a named parameter of a hazard, bound to a raster or a number
  given to `geoai run` with `--input`.
- **Kind**: what an input takes, either a `raster` or a `number`.
- **Layer**: anything with a value at each point, such as a raster read from
  a file, a raster built by `pixelwise`, or a constant.
- **Missing value**: the absence of a value at a point, because the point lies
  outside a raster, the raster holds nodata there, or the value there is not a
  finite number.
- **Model coordinates**: coordinates in the CRS of a grid, such as longitude
  and latitude, as opposed to the columns and rows of pixels.
- **Nodata**: a value that a raster stores in pixels without data, such as
  -9999.
- **PGA** (peak ground acceleration): the strongest acceleration of the
  ground that an earthquake causes at a place.  `tests/data/PGA_250y.tif`
  holds the PGA that is exceeded on average once every 250 years.
- **Pixel center**: the point in the middle of a pixel, where geoai samples
  the inputs from which it computes the pixel.
- **Pixelwise**: computed at each pixel independently, from the values at the
  same place.  The Scheme form `pixelwise` builds such a raster.
- **Raster**: a grid of pixels, each holding a number such as an elevation, a
  shaking intensity, or a hazard class.
- **Sample**: one value stored in a raster, or, as a verb, to read the value
  of a raster or layer at a point.
- **Sample type**: the type of number each pixel stores, such as `u8` for
  whole numbers from 0 to 255 or `f32` for 32-bit floating-point numbers.
- **STAC** (SpatioTemporal Asset Catalog): a standard for describing
  geospatial files as JSON records that tools can search by place and time.
- **Signature**: the first part of a hazard definition, which names the
  hazard and its inputs.
- **WGS 84**: the World Geodetic System 1984, the model of the Earth that GPS
  uses.
