// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

//! Times of runs of some work, in milliseconds, with the processor time
//! that the whole process spent on each run

use std::time::{Duration, Instant};

use cpu_time::ProcessTime;

pub struct Stats {
    /// Wall time of each run, sorted
    times: Vec<f64>,
    /// Processor time of the process for each run, on average
    pub cpu: f64,
}

impl Stats {
    pub fn new(mut times: Vec<f64>, cpu: Duration) -> Self {
        let cpu = cpu.as_secs_f64() * 1e3 / times.len().max(1) as f64;
        times.sort_by(f64::total_cmp);
        Self { times, cpu }
    }

    /// Runs `work` for about `budget`, at least five times and at most
    /// `most` times
    pub fn measure(budget: Duration, most: usize, mut work: impl FnMut()) -> Self {
        let start = Instant::now();
        let cpu = ProcessTime::now();
        let times: Vec<f64> = (0..most)
            .map_while(|index| {
                (index < 5 || start.elapsed() < budget).then(|| {
                    let began = Instant::now();
                    work();
                    began.elapsed().as_secs_f64() * 1e3
                })
            })
            .collect();
        Self::new(times, cpu.elapsed())
    }

    pub fn median(&self) -> f64 {
        self.at(0.5)
    }

    pub fn p95(&self) -> f64 {
        self.at(0.95)
    }

    fn at(&self, share: f64) -> f64 {
        let last = self.times.len().saturating_sub(1);
        self.times
            .get((last as f64 * share).round() as usize)
            .copied()
            .unwrap_or(f64::NAN)
    }
}
