// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of cues and their clips

mod clips;
mod following;
mod levels;
mod naming;
mod sections;
mod video;

use super::*;
use crate::back::sound;

/// Returns the delay between a cue firing and it firing the next cue of a
/// list that holds no group
fn next_delay(cue: &Cue) -> Option<f32> {
    cue.next_delay_in(&CueList::new("Main", Vec::new()).groups())
}

fn audio(name: &str, file: &str) -> Cue {
    Cue::new(CueId(1), "1", Kind::Audio, name)
        .file(file)
        .build()
}

fn loaded(cue: Cue) -> Cue {
    Cue {
        media: sound(10.0),
        ..cue
    }
}

fn video(start: f32, looping: bool) -> Cue {
    Cue {
        media: Media::Loaded {
            duration: 10.0,
            sound: Some(9.5),
            ahead: 0.0,
        },
        ..Cue::new(CueId(1), "1", Kind::Video, "Movie")
            .file("movie.mp4")
            .start(start)
            .looping(looping)
            .build()
    }
}

/// Returns a clip from 1 to 9 seconds into its file that loops from 3 to 5,
/// so that it reaches the end of its loop first after 4 seconds and goes
/// round the loop every 2
fn sectioned() -> Clip {
    Clip {
        start: 1.0,
        end: 9.0,
        looping: Some(Loop {
            start: 3.0,
            end: 5.0,
            released: None,
        }),
    }
}

/// Returns a clip from 2 to 5 seconds into its file, which as `looping` says
/// goes round the whole of itself
fn looping_clip(looping: bool) -> Clip {
    Clip {
        start: 2.0,
        end: 5.0,
        looping: looping.then_some(Loop {
            start: 2.0,
            end: 5.0,
            released: None,
        }),
    }
}
