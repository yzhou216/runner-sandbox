<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# Architecture

geoai computes hazard rasters from geospatial inputs and writes them as
GeoTIFF files.  Hazards are written in Scheme, so they can be added or changed
without rebuilding geoai.  The [glossary](glossary.md) explains the terms used
here.

## Modules

Each module is the file or directory of the same name under `src/`.

- `geo`: georeferencing types shared by every input and output, independent
  of any file format, and the `Layer` trait through which all raster data is
  sampled.
- `raster`: rasters held in memory, and their computation from a layer at
  every pixel of a grid.
- `storage`: files in the local filesystem, named by a path, or in an
  object store such as S3, named by a URL, whatever their format.
- `format`: one module per file format, converting between files and the
  types in `geo` and `raster`.  Files are recognized by their content when
  read and by their extension when written.
- `sexp`: S-expressions, Scheme values as plain data, the form in which
  hazard signatures and `pixelwise` expressions reach Rust.
- `pixelwise`: layers computed pixel by pixel by an expression over other
  layers, the per-pixel language of hazards.
- `hazard`: hazards as a signature, which names the inputs and describes the
  output, and a body, which computes a layer from the input values.  A hazard
  knows neither where its values come from nor the language of its body.
- `script`: the Scheme front end.  It runs each hazard file in a Steel
  engine of its own, turns each `define-hazard` into a hazard whose body is a
  Scheme procedure, and provides `pixelwise` to that procedure.
- `catalog`: hazards by name, from the built-in files in `hazards/`, the user
  directory, and paths given on the command line.
- `cli`: one module per subcommand, each a short recipe that combines the
  modules above, and `hazards` for the options they share.

`src/cli/run.rs` follows a computation from start to finish, and
`hazards/earthquake.scm` is a complete hazard.

## Data flow

```text
hazard file         --script-->  Hazard: signature and body
input files         --format-->  one Value per input
Values              --body---->  Layer
Layer, output grid  --raster-->  Raster
Raster              --format-->  GeoTIFF
```

Each stage depends only on what earlier stages produce.  The `format`
stages read and write files through `storage`, so no hazard or format knows
where a file is kept.  A new file format requires no change to any hazard,
a new kind of storage requires none to any format, a new hazard requires no
change to geoai, and another language could define hazards by implementing
`hazard::Body`.

The body runs once each time a hazard is computed and only builds a layer.
The layer is then sampled at the center of every output pixel in parallel,
and a layer built by `pixelwise` is evaluated in Rust, so no Scheme code runs
per pixel.

## Where changes belong

- A file format: a module under `src/format/`, recognized in `src/format.rs`.
- A kind of storage, such as another object store: `store` in
  `src/storage.rs`, and a feature of the `object_store` crate.
- A kind of input: a variant of `hazard::Kind` and `hazard::Value`, read in
  `src/cli/run.rs` and converted for Scheme in `src/script/primitives.rs`.
- A form of input or a declaration: a rule in the `%procedure` or
  `%define-hazard` macro of `src/script/prelude.scm`, and a case in `input`
  or `declare` of `src/hazard/definition.rs`.  The macro and the Rust code
  must accept the same forms.
- A sample type of the output: a variant of `hazard::SampleType`, an
  implementation of `format::geotiff::Sample`, and a case in
  `src/cli/run.rs`.
- A Scheme procedure implemented in Rust: `src/script/primitives.rs`.
- A per-pixel operator: `src/pixelwise/expression.rs` and
  `src/pixelwise/parse.rs`.
- A built-in hazard: a file in `hazards/`, listed in `BUILTIN` of
  `src/catalog.rs`.
