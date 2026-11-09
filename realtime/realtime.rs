// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sandbox only: plays the video file that CINQ_REALTIME_CLIP names
//! through the video output in real time, as the interface does, and
//! prints how many pictures came due in each second of the clip and how
//! long the process took on the processor

use std::{
    env,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use super::{probe, showing_cue};
use crate::back::{Command, Cue, CueId, Kind, Video};

/// Seconds of processor time that this process has used, in user and
/// kernel mode, which Bash's `time` does not report for native programs on
/// Windows
#[cfg(unix)]
fn processor_seconds() -> (f64, f64) {
    unsafe extern "C" {
        fn getrusage(who: i32, usage: *mut [u64; 36]) -> i32;
    }
    let mut usage = [0u64; 36];
    // RUSAGE_SELF is 0 everywhere; the first two timevals are user and
    // system time, whose microseconds are 32 bits wide on Darwin
    assert_eq!(unsafe { getrusage(0, &mut usage) }, 0);
    let seconds = |at: usize| usage[at] as f64 + (usage[at + 1] & 0xffff_ffff) as f64 / 1e6;
    (seconds(0), seconds(2))
}

#[cfg(windows)]
fn processor_seconds() -> (f64, f64) {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn GetProcessTimes(
            process: isize,
            created: *mut u64,
            exited: *mut u64,
            kernel: *mut u64,
            user: *mut u64,
        ) -> i32;
    }
    let (mut created, mut exited, mut kernel, mut user) = (0u64, 0u64, 0u64, 0u64);
    assert_ne!(
        unsafe {
            GetProcessTimes(
                GetCurrentProcess(),
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
        },
        0
    );
    (user as f64 / 1e7, kernel as f64 / 1e7)
}

#[test]
#[ignore = "plays the clip that CINQ_REALTIME_CLIP names, for the sandbox"]
fn plays_in_real_time() {
    let path = PathBuf::from(env::var_os("CINQ_REALTIME_CLIP").expect("a clip is named"));
    let name = path
        .file_name()
        .expect("the clip is a file")
        .to_string_lossy()
        .into_owned();
    let (duration, _) = probe(&path).expect("the clip plays");
    let workspace = showing_cue(
        &path,
        Cue::new(CueId(1), "1", Kind::Video, "Clip")
            .file(&name)
            .build(),
    );
    let mut video = Video::default();
    // Picture made ready ahead, as the GO of a show finds it
    let (workspace, _) = video.sync(workspace);
    thread::sleep(Duration::from_secs(1));
    let mut workspace = workspace.apply(Command::Go { list: 0 });
    let start = Instant::now();
    let mut last = start;
    let mut shown: Vec<f64> = Vec::new();
    let limit = Duration::from_secs_f64(f64::from(duration) + 10.0);
    while !workspace.active().is_empty() && start.elapsed() < limit {
        thread::sleep(Duration::from_millis(2));
        let now = Instant::now();
        let ticked = workspace.apply(Command::Tick((now - last).as_secs_f64()));
        last = now;
        let (synced, pictures) = video.sync(ticked);
        workspace = synced;
        shown.extend(pictures.iter().map(|_| start.elapsed().as_secs_f64()));
    }
    let seconds = (f64::from(duration).ceil() as usize).max(1);
    let each: Vec<usize> = (0..seconds)
        .map(|second| shown.iter().filter(|&&at| at as usize == second).count())
        .collect();
    let first = shown.first().copied().unwrap_or(f64::NAN);
    let (user, system) = processor_seconds();
    println!(
        "REALTIME {name}: {} pictures in {duration:.2} s, each second {each:?}, first picture {first:.3} s after the GO, cpu {user:.2} user {system:.2} system, built in {}",
        shown.len(),
        env!("CARGO_MANIFEST_DIR"),
    );
}
