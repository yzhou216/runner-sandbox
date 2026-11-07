// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

//! Times what iced's tiny-skia renderer does on the thread of the interface
//! to draw each new picture of the stage, as iced_tiny_skia 0.14.1 does it:
//! turn the RGBA of the picture into premultiplied BGRA, then draw it,
//! fitted, onto the screen with bilinear filtering.  `prescaled` measures
//! the same for a picture that worker threads have already scaled to the
//! size it shows at, which tiny-skia draws without filtering, and `scale`
//! the work of those threads with rayon's pool.

use std::time::Duration;

use rayon::prelude::*;
use tiny_skia::{Color, ColorU8, FilterQuality, Pixmap, PixmapPaint, PixmapRef, Transform};

use crate::{
    SIZES,
    color::{Way, converted},
    convert::noise,
    stats::Stats,
};

const TARGETS: [(u32, u32); 2] = [(1280, 720), (1920, 1080)];

pub fn run() {
    println!("## Drawing a new picture with tiny-skia, on the thread of the interface");
    println!();
    println!("| picture | screen | now ms | scale ms, on rayon | prescaled ms |");
    println!("| --- | --- | --- | --- | --- |");
    SIZES.iter().for_each(|&(width, height)| {
        let rgba = converted(&noise(width, height), Way::RayonStrips);
        TARGETS.iter().for_each(|&(screen_width, screen_height)| {
            let mut screen = Pixmap::new(screen_width, screen_height).expect("screen");
            let bounds = fitted((width, height), (screen_width, screen_height));
            let now = Stats::measure(Duration::from_millis(800), 200, || {
                drawn(&mut screen, &rgba, (width, height), bounds);
            });
            let size = (bounds.2 as u32, bounds.3 as u32);
            let scale = Stats::measure(Duration::from_millis(800), 200, || {
                std::hint::black_box(scaled(&rgba, (width, height), size));
            });
            let small = scaled(&rgba, (width, height), size);
            let prescaled = Stats::measure(Duration::from_millis(800), 200, || {
                drawn(&mut screen, &small, size, bounds);
            });
            println!(
                "| {width}x{height} | {screen_width}x{screen_height} | {:.2} | {:.2} | {:.2} |",
                now.median(),
                scale.median(),
                prescaled.median(),
            );
        });
    });
    println!();
}

/// Returns where a picture of `size` shows on a screen of `screen`: in the
/// middle, as large as it fits whole, on whole pixels
fn fitted(size: (u32, u32), screen: (u32, u32)) -> (f32, f32, f32, f32) {
    let (width, height) = (size.0 as f32, size.1 as f32);
    let (screen_width, screen_height) = (screen.0 as f32, screen.1 as f32);
    let scale = (screen_width / width).min(screen_height / height);
    let (fitted_width, fitted_height) = ((width * scale).round(), (height * scale).round());
    (
        ((screen_width - fitted_width) / 2.0).round(),
        ((screen_height - fitted_height) / 2.0).round(),
        fitted_width,
        fitted_height,
    )
}

/// Draws a new picture onto `screen` as iced_tiny_skia does
fn drawn(screen: &mut Pixmap, rgba: &[u8], size: (u32, u32), bounds: (f32, f32, f32, f32)) {
    let buffer: Vec<u32> = rgba
        .chunks_exact(4)
        .map(|pixel| {
            let [r, g, b, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
            bytemuck::cast(ColorU8::from_rgba(b, g, r, a).premultiply())
        })
        .collect();
    let image =
        PixmapRef::from_bytes(bytemuck::cast_slice(&buffer), size.0, size.1).expect("picture");
    screen.fill(Color::BLACK);
    let (x, y, width, height) = bounds;
    let (width_scale, height_scale) = (width / size.0 as f32, height / size.1 as f32);
    screen.draw_pixmap(
        (x / width_scale) as i32,
        (y / height_scale) as i32,
        image,
        &PixmapPaint {
            quality: FilterQuality::Bilinear,
            ..PixmapPaint::default()
        },
        Transform::identity().pre_scale(width_scale, height_scale),
        None,
    );
}

/// Returns the RGBA picture `rgba` of `size` scaled to `to` with bilinear
/// filtering, its rows spread over rayon's pool
fn scaled(rgba: &[u8], size: (u32, u32), to: (u32, u32)) -> Vec<u8> {
    let (width, height) = (size.0 as usize, size.1 as usize);
    let (to_width, to_height) = (to.0 as usize, to.1 as usize);
    let source = |to: usize, of: usize, from: usize| {
        let at = ((to as f32 + 0.5) * from as f32 / of as f32 - 0.5).clamp(0.0, (from - 1) as f32);
        let first = at as usize;
        (first, (first + 1).min(from - 1), at - first as f32)
    };
    let mut pixels = vec![0; to_width * to_height * 4];
    pixels
        .par_chunks_mut(to_width * 4)
        .enumerate()
        .for_each(|(row, line)| {
            let (top, bottom, down) = source(row, to_height, height);
            line.chunks_exact_mut(4).enumerate().for_each(|(column, pixel)| {
                let (left, right, across) = source(column, to_width, width);
                let at = |y: usize, x: usize, channel: usize| f32::from(rgba[(y * width + x) * 4 + channel]);
                pixel.iter_mut().enumerate().for_each(|(channel, value)| {
                    let upper = at(top, left, channel) * (1.0 - across) + at(top, right, channel) * across;
                    let lower =
                        at(bottom, left, channel) * (1.0 - across) + at(bottom, right, channel) * across;
                    *value = (upper * (1.0 - down) + lower * down + 0.5) as u8;
                });
            });
        });
    pixels
}
