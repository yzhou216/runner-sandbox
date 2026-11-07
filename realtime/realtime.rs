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

use super::showing_cue;
use crate::back::{Command, Cue, CueId, Kind, Video, video::probe};

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
        .map(|second| {
            shown
                .iter()
                .filter(|&&at| at as usize == second)
                .count()
        })
        .collect();
    println!(
        "REALTIME {name}: {} pictures in {duration:.2} s, each second {each:?}",
        shown.len()
    );
}
