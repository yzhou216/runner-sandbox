// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

//! Times turning one decoded picture into RGBA each way, back to back and
//! paced at sixty pictures a second, as a video plays

use std::time::{Duration, Instant};

use cpu_time::ProcessTime;
use rust_h264::decoder::{ChromaFormat, Frame};

use crate::{
    SIZES,
    color::{Kernel, STRIPS, Way, convert_into, converted, converted_with},
    stats::Stats,
};

/// Picture of `width` by `height` with planes of noise
pub fn noise(width: u32, height: u32) -> Frame {
    let (w, h) = (width as usize, height as usize);
    let plane = |length: usize, seed: u32| {
        (0..length)
            .scan(seed, |state, _| {
                *state ^= *state << 13;
                *state ^= *state >> 17;
                *state ^= *state << 5;
                Some((*state >> 24) as u8)
            })
            .collect::<Vec<u8>>()
    };
    Frame {
        width,
        height,
        y: plane(w * h, 1),
        u: plane(w.div_ceil(2) * h.div_ceil(2), 2),
        v: plane(w.div_ceil(2) * h.div_ceil(2), 3),
        pic_order_cnt: 0,
        chroma_format: ChromaFormat::Yuv420,
        bit_depth: 8,
    }
}

pub fn run() {
    println!("## Conversion to RGBA, {} strips", *STRIPS);
    println!();
    println!(
        "| size | way | burst ms | burst CPU ms | paced median ms | paced p95 ms | paced CPU ms |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    SIZES.iter().for_each(|&(width, height)| {
        let picture = noise(width, height);
        // Warms the pool and the allocator
        Way::ALL.iter().for_each(|&way| drop(converted(&picture, way)));
        Way::ALL.iter().for_each(|&way| {
            let burst = burst(&picture, way);
            let paced = paced(&picture, way);
            println!(
                "| {width}x{height} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
                way.name(),
                burst.median(),
                burst.cpu,
                paced.median(),
                paced.p95(),
                paced.cpu,
            );
        });
        // Same in a buffer that is already in memory, which tells what
        // allocating and faulting in a new one costs
        let mut reused = vec![0; width as usize * height as usize * 4];
        let stats = Stats::measure(Duration::from_millis(800), 400, || {
            convert_into(&picture, Way::RayonStrips, &mut reused);
        });
        println!(
            "| {width}x{height} | rayon-strips, buffer reused | {:.2} | {:.2} | | | |",
            stats.median(),
            stats.cpu
        );
    });
    println!();
}

fn burst(picture: &Frame, way: Way) -> Stats {
    Stats::measure(Duration::from_millis(800), 400, || {
        std::hint::black_box(converted(picture, way));
    })
}

/// Converts 90 pictures at sixty a second, as a video plays, so that the
/// threads of a pool go to sleep between them
fn paced(picture: &Frame, way: Way) -> Stats {
    let period = Duration::from_secs_f64(1.0 / 60.0);
    let start = Instant::now();
    let cpu = ProcessTime::now();
    let times: Vec<f64> = (0..90u32)
        .map(|index| {
            let due = start + period * index;
            if let Some(wait) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(wait);
            }
            let began = Instant::now();
            std::hint::black_box(converted(picture, way));
            began.elapsed().as_secs_f64() * 1e3
        })
        .collect();
    Stats::new(times, cpu.elapsed())
}

/// Prints how long each kernel takes, alone and on rayon's pool, and
/// whether it gives the bytes that cinq's does, at even sizes and odd ones
pub fn kernels() {
    println!("## Kernels of the conversion, {} strips", *STRIPS);
    println!();
    let odd = noise(641, 361);
    let reference = converted_with(&odd, Way::Serial, Kernel::Now);
    Kernel::ALL.iter().for_each(|&kernel| {
        let same = [Way::Serial, Way::RayonStrips]
            .iter()
            .all(|&way| converted_with(&odd, way, kernel) == reference);
        println!("- {} at 641x361: {}", kernel.name(), if same { "same bytes" } else { "DIFFERENT BYTES" });
    });
    println!();
    println!("| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |");
    println!("| --- | --- | --- | --- | --- | --- |");
    SIZES.iter().for_each(|&(width, height)| {
        let picture = noise(width, height);
        let reference = converted_with(&picture, Way::Serial, Kernel::Now);
        Kernel::ALL.iter().for_each(|&kernel| {
            let same = converted_with(&picture, Way::RayonStrips, kernel) == reference;
            let serial = Stats::measure(Duration::from_millis(800), 400, || {
                std::hint::black_box(converted_with(&picture, Way::Serial, kernel));
            });
            let pooled = Stats::measure(Duration::from_millis(800), 400, || {
                std::hint::black_box(converted_with(&picture, Way::RayonStrips, kernel));
            });
            println!(
                "| {width}x{height} | {} | {} | {:.2} | {:.2} | {:.2} |",
                kernel.name(),
                if same { "yes" } else { "NO" },
                serial.median(),
                pooled.median(),
                pooled.cpu,
            );
        });
    });
    println!();
}
