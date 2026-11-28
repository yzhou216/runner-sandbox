// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the text for the chain that the next GO fires

use super::*;

#[test]
fn a_chain_with_nothing_to_fire_says_so_and_names_each_cue_with_its_reason() {
    let audio = |id: u32| {
        Cue::new(CueId(id), id.to_string(), Kind::Audio, "Rain")
            .advance(Advance::Continue)
            .build()
    };

    let words = |cues: Vec<Cue>| {
        let workspace = fresh(cues);
        let list = &workspace.lists()[0];
        let standby = workspace
            .cue(list.standby.expect("a standby cue"))
            .expect("it exists");
        chain(&workspace, list, standby, &[])
    };
    assert_eq!(
        words(vec![audio(1)]),
        "GO fires nothing: cue 1 (no file set)"
    );
    assert_eq!(
        words(vec![audio(1), audio(2)]),
        "GO fires nothing: cue 1 (no file set), cue 2 (no file set)"
    );
    assert_eq!(words(vec![memo(3)]), "GO fires 1 cue: 3");
}

#[test]
fn a_chain_tells_a_cue_that_plays_in_silence_and_still_fires_it() {
    let audio = |id: u32, sends: Sends| Cue {
        sends: Some(sends),
        ..Cue::new(CueId(id), id.to_string(), Kind::Audio, "Rain")
            .file("sound.ogg")
            .advance(Advance::Continue)
            .build()
    };
    let workspace = fresh(vec![
        audio(1, [(OutputId(1), 0.0)].into()),
        audio(2, [(OutputId(9), 0.0)].into()),
    ]);
    let workspace = [1, 2]
        .into_iter()
        .map(|id| loaded(id, "sound.ogg", sound(10.0)))
        .fold(workspace, Workspace::apply);
    assert_eq!(
        chained(&workspace, 1, &[]),
        "GO fires 2 cues: 1, 2 (+0.0s, silent: no output)"
    );
    assert_eq!(
        chained(&workspace, 1, &[OutputId(1)]),
        "GO fires 2 cues: 1 (silent: output failed), 2 (+0.0s, silent: no output)"
    );
}

#[test]
fn a_chain_tells_what_each_control_cue_does_and_names_a_list_only_when_it_is_another() {
    let control = |id: u32, kind, target: u32| {
        Cue::new(CueId(id), id.to_string(), kind, "Control")
            .advance(Advance::Continue)
            .target(CueId(target))
            .build()
    };
    let effect = Cue::new(CueId(30), "E1", Kind::Memo, "Rain").build();
    let workspace = Workspace::new(vec![
        CueList::new(
            "Scene",
            vec![
                memo(1),
                control(12, Kind::Pause, 1),
                control(13, Kind::Stop, 1),
                control(15, Kind::Start, 30),
                control(16, Kind::GoTo, 1),
            ],
        ),
        CueList::new("Effects", vec![effect]),
    ])
    .apply(Command::Standby {
        list: 0,
        cue: Some(CueId(12)),
    });
    assert_eq!(
        chained(&workspace, 12, &[]),
        "GO fires 4 cues: 12 (pause 1), 13 (+0.0s, stop 1), \
         15 (+0.0s, start E1 in Effects), 16 (+0.0s, GoTo 1)"
    );
}

#[test]
fn a_chain_tells_a_delay_of_a_minute_or_more_as_a_clock_as_the_moving_standby_does() {
    let held = Cue {
        post_wait: 70.0.into(),
        ..Cue::new(CueId(1), "1", Kind::Memo, "Note")
            .advance(Advance::Continue)
            .build()
    };
    let workspace = fresh(vec![held, memo(2)]);
    assert_eq!(
        chained(&workspace, 1, &[]),
        "GO fires 2 cues: 1, 2 (+1:10.0)"
    );
}
