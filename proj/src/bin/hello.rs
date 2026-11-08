// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

fn main() {
    let start = Instant::now();
    println!("hello from Rust std");
    let shared = Arc::new(Mutex::new(HashMap::new()));
    let threads: Vec<_> = (0..4)
        .map(|i| {
            let shared = Arc::clone(&shared);
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(50));
                shared.lock().unwrap().insert(i, i * i);
            })
        })
        .collect();
    threads.into_iter().for_each(|t| t.join().unwrap());
    println!("threads: {:?}", shared.lock().unwrap().len());
    let path = std::env::temp_dir().join("rostest.txt");
    std::fs::write(&path, b"written").unwrap();
    println!("file: {:?} {:?}", path, std::fs::read_to_string(&path).unwrap());
    println!("canonical: {:?}", std::fs::canonicalize(&path));
    println!("metadata: {:?}", std::fs::metadata(&path).map(|m| m.len()));
    println!("time: {:?} elapsed {:?}", SystemTime::now(), start.elapsed());
}
