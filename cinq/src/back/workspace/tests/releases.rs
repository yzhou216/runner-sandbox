// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests that shows as the first and the fifth release wrote them still open
//! with their meaning

use super::*;
use crate::back::{Channels, Device, Instrument, InstrumentId, Output, OutputId};

/// Checks that a show has no problem but files that have not loaded, and
/// that it writes back as the same show
fn opens(read: &Workspace) {
    let broken: Vec<_> = problems(read)
        .into_iter()
        .filter(|(_, problem)| *problem != Problem::NotLoaded)
        .collect();
    assert!(broken.is_empty(), "{broken:?}");
    read_back(&shown(read));
}

/// Show as the first release wrote it, before outputs, lights, and loop
/// sections, with every setting that it wrote, and a target that it kept on
/// a cue that takes none
const FIRST_RELEASE: &str = r#"[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Rain"
file = "rain.ogg"
start = 1.5
end = 20.0
loop = true
target = 9
pre_wait = 0.5
post_wait = 2.0
advance = "continue"
level = -6.0
notes = "kept"

[[list.cue]]
id = 2
kind = "fade"
target = 1
duration = 8.0
level = -60.0
stop_target = false
"#;

#[test]
fn show_of_the_first_release_reads_as_it_did() {
    let read = Workspace::from_show(FIRST_RELEASE, "/show").expect("the show reads");
    let cue = |id, kind: Kind| Cue::new(CueId(id), "", kind, "");
    assert_eq!(
        read.lists()[0].cues,
        [
            Cue::new(CueId(1), "1", Kind::Audio, "Rain")
                .file("rain.ogg")
                .start(1.5)
                .end(20.0)
                .looping(true)
                .target(CueId(9))
                .pre_wait(0.5)
                .post_wait(2.0)
                .advance(Advance::Continue)
                .level(-6.0)
                .notes("kept")
                .build(),
            cue(2, Kind::Fade)
                .target(CueId(1))
                .duration(8.0)
                .stop_target(false)
                .build(),
        ]
    );
    assert_eq!(read.outputs(), [Output::main(OutputId(1))]);
    assert_eq!(
        read.sends(&read.lists()[0].cues[0]),
        [(OutputId(1), 0.0)].into()
    );
    opens(&read);
}

/// Show as the fifth release wrote it, with outputs, a patch, and lights
const FIFTH_RELEASE: &str = r#"[[output]]
name = "House"
device = "alsa:1"
device_name = "Card"
channels = "3-4"
new_cues = -3.0

[[output]]
name = "Sub"
channels = "7"

[[instrument]]
id = 4
name = "Wash"
universe = 2
address = 17

[[list]]
name = "Main"

[[list.cue]]
id = 1
kind = "audio"
file = "a.ogg"
level = 0.0

[list.cue.sends]
House = -3.0

[[list.cue]]
id = 2
kind = "light"
duration = 5.0

[list.cue.lights]
4 = 75.0

[[list.cue]]
id = 3
kind = "network"
destination = "udp://10.0.0.7:8000"
message = "/go 1"
"#;

#[test]
fn show_of_the_fifth_release_reads_as_it_did() {
    let read = Workspace::from_show(FIFTH_RELEASE, "/show").expect("the show reads");
    let cue = |id, kind: Kind| Cue::new(CueId(id), "", kind, "");
    assert_eq!(
        read.lists()[0].cues,
        [
            cue(1, Kind::Audio)
                .file("a.ogg")
                .sends([(OutputId(1), -3.0)].into())
                .build(),
            cue(2, Kind::Light)
                .lights([(InstrumentId(4), 75.0)].into())
                .duration(5.0)
                .build(),
            cue(3, Kind::Network)
                .destination("udp://10.0.0.7:8000")
                .message("/go 1")
                .build(),
        ]
    );
    assert_eq!(
        read.outputs(),
        [
            Output {
                id: OutputId(1),
                name: "House".to_owned(),
                device: Device::Named {
                    id: "alsa:1".to_owned(),
                    name: "Card".to_owned(),
                },
                channels: Channels::Stereo(2, 3),
                new_cues: Some(-3.0),
            },
            Output {
                id: OutputId(2),
                name: "Sub".to_owned(),
                device: Device::Default,
                channels: Channels::Mono(6),
                new_cues: None,
            },
        ]
    );
    assert_eq!(
        read.patch(),
        [Instrument {
            id: InstrumentId(4),
            name: "Wash".to_owned(),
            universe: 2,
            address: 17,
        }]
    );
    assert_eq!(read.unlit_cues().count(), 0);
    opens(&read);
}
