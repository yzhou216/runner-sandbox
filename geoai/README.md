<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# geoai

geoai computes hazard rasters, such as earthquake hazard classes, from
geospatial inputs and writes them as GeoTIFF files.  Hazards are defined in
Scheme, so adding one requires no change to geoai.

## Getting started

Building requires a Rust toolchain, and the test data requires Git LFS.

```sh
git lfs pull
cargo run --release -- list
cargo run --release -- run earthquake earthquake.tif \
  --input pga=tests/data/PGA_250y.tif
cargo run --release -- info earthquake.tif
cargo test
```

`nix flake check` runs the checks of the pre-push hook in `.githooks`: Clippy,
the tests, and the formatting, Markdown, and licensing checks.  `nix develop`
enables the hook and provides Git LFS and the Markdown and licensing tools.

`cargo xtask man target/man` writes the manual pages, one for geoai and one
for each command, and `nix build` includes them in the package.

## Documentation

- [Hazards](doc/hazards.md): running hazards and defining new ones.
- [Storage](doc/storage.md): files in object stores such as S3, and how
  the geoai service keeps rasters and their metadata.
- [Architecture](doc/architecture.md): the modules, how data flows between
  them, and where each kind of change belongs.
- [Glossary](doc/glossary.md): the terms of geoai and of geospatial work
  used throughout.

## License

The source code is licensed under the GNU Affero General Public License,
version 3 or later, and the documentation and build files under 0BSD or
CC0-1.0.  Each file declares its license following the REUSE specification.
