// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests that a show with a cue of every kind, and every setting that its
//! kind uses, reads back from its file as it was

use super::*;
use crate::back::{Channels, Device, Instrument, InstrumentId, Output, OutputId, Sends};

fn instrument(id: u32, universe: u16, address: u16) -> Instrument {
    Instrument {
        id: InstrumentId(id),
        name: format!("Par \"{id}\""),
        universe,
        address,
    }
}

/// Returns a show with a cue of every kind, one with a loop section that
/// it does not loop, an empty list, two outputs, and two instruments
fn everything() -> Workspace {
    let sends = [(OutputId(1), -3.5), (OutputId(2), -12.0)]
        .into_iter()
        .collect();
    let acting = |id: u32, kind: Kind| {
        Cue::new(CueId(id), id.to_string(), kind, format!("{kind} cue"))
            .target(CueId(1))
            .pre_wait(0.25)
            .post_wait(1.5)
            .advance(Advance::Follow)
    };
    let cues = vec![
        Cue::new(
            CueId(1),
            "1.5",
            Kind::Audio,
            "Rain \"loud\"\nand\tsoft \\ \u{1}",
        )
        .file("sounds/rain.ogg")
        .start(1.5)
        .end(20.0)
        .looping(true)
        .loop_start(3.0)
        .loop_end(10.0)
        .level(-6.5)
        .sends(sends)
        .pre_wait(0.5)
        .post_wait(2.0)
        .advance(Advance::Continue)
        .notes("two\nlines \u{2028} and a note")
        .build(),
        Cue::new(CueId(2), "", Kind::Video, "Movie")
            .file("movie.mp4")
            .start(1.5)
            .looping(true)
            .sends(Sends::default())
            .build(),
        Cue::new(CueId(3), "3", Kind::Picture, "Slide")
            .file("slide.png")
            .build(),
        Cue::new(CueId(4), "4", Kind::Light, "Wash")
            .lights([(InstrumentId(1), 50.0), (InstrumentId(2), 100.0)].into())
            .duration(3.25)
            .build(),
        acting(5, Kind::Fade)
            .level(-20.0)
            .duration(4.0)
            .stop_target(false)
            .build(),
        acting(6, Kind::Start).build(),
        acting(7, Kind::Stop).build(),
        acting(8, Kind::Pause).build(),
        acting(9, Kind::Devamp).build(),
        acting(10, Kind::GoTo).build(),
        Cue::new(CueId(11), "11", Kind::Network, "Console")
            .destination("10.101.100.20:3037")
            .message("/eos/cue/1/fire")
            .build(),
        Cue::new(CueId(12), "12", Kind::Group, "Group")
            .collapsed(true)
            .build(),
        Cue::new(CueId(13), "", Kind::Wait, "Hold")
            .duration(2.5)
            .depth(1)
            .build(),
        Cue::new(CueId(14), "14", Kind::Memo, "Note")
            .notes("remember")
            .depth(1)
            .build(),
    ];
    let vamp = Cue::new(CueId(15), "", Kind::Audio, "Vamp")
        .file("vamp.ogg")
        .loop_start(2.0)
        .loop_end(6.0)
        .build();
    Workspace::new(vec![
        CueList::new("Main", cues),
        CueList::new("Pre\"show", vec![vamp]),
        CueList::new("Empty", Vec::new()),
    ])
    .directory("/show")
    .with_outputs(vec![
        Output {
            id: OutputId(1),
            name: "House \"L\"".to_owned(),
            device: Device::Named {
                id: "alsa:1".to_owned(),
                name: "Interface".to_owned(),
            },
            channels: Channels::Stereo(2, 3),
            new_cues: Some(-6.0),
        },
        Output {
            id: OutputId(2),
            name: "Sub.woofer".to_owned(),
            device: Device::Default,
            channels: Channels::Mono(7),
            new_cues: None,
        },
    ])
    .with_patch(vec![instrument(1, 1, 1), instrument(2, 3, 512)])
}

#[test]
fn show_with_every_kind_of_cue_reads_back_as_it_was() {
    let saved = everything();
    let text = shown(&saved);
    let read = read_back(&text);
    assert_eq!(read.outputs(), saved.outputs());
    assert_eq!(read.patch(), saved.patch());
    assert_eq!(read.next_cue_id(), Some(CueId(16)));
    assert_eq!(read.lists().len(), saved.lists().len());
    read.lists()
        .iter()
        .zip(saved.lists())
        .for_each(|(read, saved)| {
            assert_eq!((&read.name, read.standby), (&saved.name, saved.standby));
            assert_eq!(read.cues, saved.cues);
        });
    // Keys that a file keeps in the order that its manual page shows
    [
        "file = \"sounds/rain.ogg\"\nstart = 1.5\nend = 20.0\nloop = true\nloop_start = 3.0\nloop_end = 10.0\n",
        "kind = \"video\"\nname = \"Movie\"\nfile = \"movie.mp4\"\nstart = 1.5\nloop = true\nlevel = 0.0\n",
        "kind = \"devamp\"\nname = \"Devamp cue\"\ntarget = 1\n",
        "destination = \"10.101.100.20:3037\"\nmessage = \"/eos/cue/1/fire\"\n",
    ]
    .into_iter()
    .for_each(|keys| assert!(text.contains(keys), "{keys}\n{text}"));
}
