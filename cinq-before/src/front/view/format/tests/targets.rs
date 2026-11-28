// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how the target of a cue is told

use super::*;

#[test]
fn target_goes_by_its_number_or_else_by_its_quoted_name() {
    let fade = |target: u32| {
        Cue::new(CueId(10 + target), "10", Kind::Fade, "Fade")
            .target(CueId(target))
            .build()
    };
    let numbered = Cue::new(CueId(1), "1", Kind::Audio, "Rain").build();
    let unnumbered = Cue::new(CueId(2), "", Kind::Audio, "Thunder").build();
    let workspace = fresh(vec![numbered, unnumbered]);
    let groups = &workspace.lists()[0].groups();
    assert_eq!(target(&workspace, groups, &fade(1)), "1");
    assert_eq!(target(&workspace, groups, &fade(2)), "\"Thunder\"");
    assert_eq!(target(&workspace, groups, &fade(3)), "");
}

#[test]
fn target_in_another_list_is_told_by_the_name_of_that_list() {
    let start = |number: &str, target: u32| {
        Cue::new(CueId(40 + target), number, Kind::Start, "Start")
            .target(CueId(target))
            .build()
    };
    let workspace = Workspace::new(vec![
        CueList::new(
            "Main",
            vec![memo(1), start("2", 1), start("3", 9), start("4", 99)],
        ),
        CueList::new("Effects", vec![memo(9)]),
    ]);
    let groups = &workspace.lists()[0].groups();
    let told = |id: u32| {
        let cue = workspace.cue(CueId(id)).expect("it exists");
        target(&workspace, groups, cue)
    };
    assert_eq!(told(41), "1");
    assert_eq!(told(49), "9 in Effects");
    assert_eq!(told(139), "");
}

#[test]
fn a_light_cue_counts_every_instrument_it_names_even_one_the_patch_lacks() {
    let light = |ids: &[u32]| {
        Cue::new(CueId(1), "1", Kind::Light, "Light")
            .lights(ids.iter().map(|&id| (InstrumentId(id), 50.0)).collect())
            .build()
    };
    let workspace = fresh(vec![]);
    let groups = &workspace.lists()[0].groups();
    assert_eq!(target(&workspace, groups, &light(&[99])), "1 instrument");
    assert_eq!(
        target(&workspace, groups, &light(&[1, 99])),
        "2 instruments"
    );
    assert_eq!(target(&workspace, groups, &light(&[])), "0 instruments");
}
