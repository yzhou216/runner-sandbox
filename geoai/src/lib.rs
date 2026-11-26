// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Geospatial hazard processing.
//!
//! Each [`format`](mod@format) module converts one file format to and from
//! the types of [`geo`] and [`raster`], reading and writing files wherever
//! [`storage`] keeps them.  A [`hazard`] declares its inputs and
//! leaves the computation to a body, which receives the input values and
//! returns a [`geo::Layer`], so it never touches files.  Bodies are written in
//! Scheme and run by [`script`], usually returning a [`pixelwise`] layer that
//! is computed natively at every pixel.  Hazards are found by name in the
//! [`catalog`].

pub mod catalog;
pub mod cli;
pub mod format;
pub mod geo;
pub mod hazard;
pub mod pixelwise;
pub mod raster;
pub mod script;
pub mod sexp;
pub mod storage;
