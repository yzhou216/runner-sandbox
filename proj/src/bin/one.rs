// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// Rust's standard library on one thread, which exits through the TLS
// callback of process detach alone

use std::collections::HashMap;
use std::time::{Instant, SystemTime};

fn main() {
    let start = Instant::now();
    let squares: HashMap<u32, u32> = (0..4).map(|i| (i, i * i)).collect();
    println!("hello from one thread: {squares:?}");
    let path = std::env::temp_dir().join("rostest1.txt");
    std::fs::write(&path, b"written").unwrap();
    println!("file: {:?} {:?}", path, std::fs::read_to_string(&path));
    println!("time: {:?} elapsed {:?}", SystemTime::now(), start.elapsed());
    // C runtime math, which ReactOS's ucrtbase.dll partly stubs
    let x = std::hint::black_box(2.7f32);
    println!(
        "math: trunc {} {} ln_1p {:e} asinh {} cbrt {}",
        x.trunc(),
        (-x).trunc(),
        std::hint::black_box(1e-10f64).ln_1p(),
        std::hint::black_box(-2.0f64).asinh(),
        std::hint::black_box(27.0f32).cbrt(),
    );
}
