// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the text that describes cues

use super::{change, target};
use crate::back::{Command, Cue, CueId, CueList, Kind, Place, Workspace};

#[test]
fn target_goes_by_its_number_or_else_by_its_quoted_name() {
    let fade = |target: u32| {
        Cue::new(CueId(10 + target), "10", Kind::Fade, "Fade")
            .target(CueId(target))
            .build()
    };
    let numbered = Cue::new(CueId(1), "1", Kind::Audio, "Rain").build();
    let unnumbered = Cue::new(CueId(2), "", Kind::Audio, "Thunder").build();
    let workspace = Workspace::new(vec![CueList::new("Main", vec![numbered, unnumbered])]);
    assert_eq!(target(&workspace, &fade(1)), "1");
    assert_eq!(target(&workspace, &fade(2)), "\"Thunder\"");
    assert_eq!(target(&workspace, &fade(3)), "");
}

#[test]
fn pasted_cues_are_told_by_the_number_of_the_first_or_else_by_how_many() {
    let memo = |id: u32| Cue::new(CueId(id), id.to_string(), Kind::Memo, "Note").build();
    let shown = Workspace::new(vec![CueList::new("Main", vec![memo(1)])]);
    let told = |count: usize| {
        let pasted = shown.clone().apply(Command::Paste {
            list: 0,
            cues: vec![memo(1); count],
            at: Place::End,
        });
        let (_, latest) = pasted.latest_change().expect("a paste is a change");
        change(&pasted, latest)
    };
    assert_eq!(told(1), "pasted cue 2");
    assert_eq!(told(3), "pasted 3 cues");
}
