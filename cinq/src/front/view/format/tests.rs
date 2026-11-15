// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the text that describes cues

use super::{change, clipped, target};
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

#[test]
fn copied_and_cut_cues_are_told_by_the_number_of_the_cue_or_else_by_how_many() {
    let memo = |id: u32| Cue::new(CueId(id), id.to_string(), Kind::Memo, "Note").build();
    assert_eq!(clipped("Copied", &[memo(4)]), "Copied cue 4");
    assert_eq!(clipped("Cut", &[memo(1), memo(2), memo(3)]), "Cut 3 cues");
}

#[test]
fn added_cues_are_told_by_the_number_of_the_one_or_else_by_how_many() {
    let shown = Workspace::new(vec![CueList::new("Main", Vec::new())]);
    let told = |count: usize| {
        let added = shown.clone().apply(Command::Add {
            list: 0,
            kind: Kind::Memo,
            at: Place::End,
            count,
        });
        let (_, latest) = added.latest_change().expect("an add is a change");
        change(&added, latest)
    };
    assert_eq!(told(1), "added cue 1");
    assert_eq!(told(3), "added 3 cues");
}

#[test]
fn cut_cues_stay_cut_in_the_words_of_the_change_and_deleted_ones_stay_deleted() {
    let memo = |id: u32| Cue::new(CueId(id), id.to_string(), Kind::Memo, "Note").build();
    let shown = Workspace::new(vec![CueList::new("Main", vec![memo(1), memo(2), memo(3)])]);
    let told = |command: fn(Vec<CueId>) -> Command, ids: &[u32]| {
        let removed = shown
            .clone()
            .apply(command(ids.iter().map(|&id| CueId(id)).collect()));
        let (_, latest) = removed.latest_change().expect("a removal is a change");
        change(&removed, latest)
    };
    assert_eq!(told(Command::Cut, &[2]), "cut cue 2");
    assert_eq!(told(Command::Cut, &[1, 2]), "cut 2 cues");
    assert_eq!(told(Command::Delete, &[2]), "deleted cue 2");
    assert_eq!(told(Command::Delete, &[1, 2]), "deleted 2 cues");
}
