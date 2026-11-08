// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// One kind of thread of Rust's standard library, named by the argument,
// started and joined, to find what makes a thread's exit fail

use std::sync::Mutex;
use std::time::Duration;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    eprintln!("threads: {mode}");
    let lock = Mutex::new(0);
    std::thread::scope(|scope| {
        let work = || match mode.as_str() {
            "empty" => {}
            "sleep" => std::thread::sleep(Duration::from_millis(50)),
            "mutex" => *lock.lock().unwrap() += 1,
            "print" => eprintln!("threads: in {:?}", std::thread::current().id()),
            "local" => {
                thread_local!(static NAME: String = String::from("local"));
                NAME.with(|name| eprintln!("threads: {name}"));
            }
            "drop" => {
                struct Loud;
                impl Drop for Loud {
                    fn drop(&mut self) {
                        eprintln!("threads: destructor ran");
                    }
                }
                thread_local!(static LOUD: Loud = const { Loud });
                LOUD.with(|_| ());
            }
            other => panic!("unknown mode {other}"),
        };
        let thread = scope.spawn(work);
        thread.join().unwrap();
        eprintln!("threads: one joined");
        let threads: Vec<_> = (0..4).map(|_| scope.spawn(work)).collect();
        threads.into_iter().for_each(|thread| thread.join().unwrap());
        eprintln!("threads: four joined");
    });
}
