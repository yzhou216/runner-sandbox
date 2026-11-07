// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

//! Benchmarks of the parts of cinq's video output that use the processor
//! most, each the way cinq does it now and the ways that rayon and more
//! threads could: `cinq-bench convert`, `cinq-bench draw`, and
//! `cinq-bench pass CLIP...`, each of which prints a Markdown table

mod color;
mod convert;
// Conversion of the snapshot of cinq in the sandbox, as cinq builds it
#[path = "../../cinq/src/back/video/decoder/color.rs"]
#[allow(dead_code)]
mod snapshot;
mod draw;
mod pass;
mod stats;

/// Sizes of the pictures timed
pub const SIZES: [(u32, u32); 4] = [(640, 360), (1280, 720), (1920, 1080), (3840, 2160)];

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.split_first() {
        Some((command, _)) if command == "convert" => convert::run(),
        Some((command, _)) if command == "draw" => draw::run(),
        Some((command, _)) if command == "kernels" => {
            convert::kernels();
            convert::snapshot();
        }
        Some((command, clips)) if command == "pass" => pass::run(clips),
        _ => {
            eprintln!("usage: cinq-bench convert | kernels | draw | pass CLIP...");
            std::process::exit(2);
        }
    }
}
