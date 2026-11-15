// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the edits that change one setting of a cue and that take it back

use super::{Advance, Cue, CueId, Edit};
use crate::back::{Kind, Media};

/// Returns a cue whose every setting differs from what the edits below set
fn cue() -> Cue {
    Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .file("rain.ogg")
        .start(1.0)
        .end(9.0)
        .looping(true)
        .target(CueId(7))
        .pre_wait(2.0)
        .duration(3.0)
        .post_wait(4.0)
        .advance(Advance::Continue)
        .notes("Outdoors")
        .level(-6.0)
        .stop_target(true)
        .build()
}

/// Returns an edit of each setting, each with a value other than the cue's
fn edits() -> Vec<Edit> {
    vec![
        Edit::Number("2".to_owned()),
        Edit::Name("Storm".to_owned()),
        Edit::File("storm.ogg".to_owned()),
        Edit::FileAndName {
            file: "storm.ogg".to_owned(),
            name: "Storm".to_owned(),
        },
        Edit::Start(2.5.into()),
        Edit::End(8.5.into()),
        Edit::Loop(false),
        Edit::Target(None),
        Edit::Notes("Indoors".to_owned()),
        Edit::PreWait(0.5.into()),
        Edit::Duration(6.0.into()),
        Edit::PostWait(1.5.into()),
        Edit::Advance(Advance::Follow),
        Edit::Level(-12.0),
        Edit::StopTarget(false),
    ]
}

#[test]
fn edit_changes_the_setting_it_names_and_no_other() {
    let changed = |edit: Edit| cue().edited(edit);
    let wanted = [
        Cue {
            number: "2".to_owned(),
            ..cue()
        },
        Cue {
            name: "Storm".to_owned(),
            ..cue()
        },
        Cue {
            file: "storm.ogg".to_owned(),
            ..cue()
        },
        Cue {
            file: "storm.ogg".to_owned(),
            name: "Storm".to_owned(),
            ..cue()
        },
        Cue {
            start: 2.5.into(),
            ..cue()
        },
        Cue {
            end: 8.5.into(),
            ..cue()
        },
        Cue {
            looping: false,
            ..cue()
        },
        Cue {
            target: None,
            ..cue()
        },
        Cue {
            notes: "Indoors".to_owned(),
            ..cue()
        },
        Cue {
            pre_wait: 0.5.into(),
            ..cue()
        },
        Cue {
            duration: 6.0.into(),
            ..cue()
        },
        Cue {
            post_wait: 1.5.into(),
            ..cue()
        },
        Cue {
            advance: Advance::Follow,
            ..cue()
        },
        Cue {
            level: -12.0,
            ..cue()
        },
        Cue {
            stop_target: false,
            ..cue()
        },
    ];
    let changed: Vec<Cue> = edits().into_iter().map(changed).collect();
    assert_eq!(changed, wanted);
}

#[test]
fn reverting_an_edit_sets_the_setting_back_to_what_the_cue_held() {
    let left_changed: Vec<Edit> = edits()
        .into_iter()
        .filter(|edit| {
            let revert = cue().reverting(edit);
            cue().edited(edit.clone()).edited(revert) != cue()
        })
        .collect();
    assert_eq!(left_changed, []);
}

#[test]
fn edit_that_changes_the_file_does_not_touch_the_media_of_the_cue() {
    let loaded = Cue {
        media: Media::Loaded {
            duration: 10.0,
            sound: Some(10.0),
        },
        ..cue()
    };
    let edited = loaded.clone().edited(Edit::File("storm.ogg".to_owned()));
    assert_eq!(edited.media, loaded.media);
}
