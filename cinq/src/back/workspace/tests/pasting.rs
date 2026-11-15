// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of pasting copies of cues, which take new identities and keep
//! their settings, and of what a copy keeps of its file and its target

use super::*;

fn paste(cues: Vec<Cue>, at: Place) -> Command {
    Command::Paste { list: 0, cues, at }
}

/// Returns copies of the cues of `from` with the identities `ids`
fn copies(from: &Workspace, ids: &[u32]) -> Vec<Cue> {
    ids.iter()
        .filter_map(|&id| from.cue(CueId(id)))
        .map(|cue| from.copied(cue))
        .collect()
}

#[test]
fn pasted_cues_take_new_identities_and_keep_their_settings_in_order() {
    let shown = workspace(vec![audio(1), audio(2), audio(3)]);
    let cues = copies(&shown, &[1, 2]);
    let pasted = shown.apply(paste(cues, Place::After(CueId(3))));
    assert_eq!(ids(&pasted), [1, 2, 3, 4, 5]);
    let (copy, original) = (pasted.cue(CueId(4)).unwrap(), pasted.cue(CueId(1)).unwrap());
    assert_eq!(
        (&copy.name, &copy.file, copy.kind),
        (&original.name, &original.file, original.kind)
    );
}

#[test]
fn pasting_goes_below_or_above_a_cue_or_else_to_the_end() {
    let shown = workspace(vec![audio(1), audio(2)]);
    let cue = || copies(&shown, &[1]);
    let after = shown.clone().apply(paste(cue(), Place::After(CueId(1))));
    assert_eq!(ids(&after), [1, 3, 2]);
    let before = shown.clone().apply(paste(cue(), Place::Before(CueId(1))));
    assert_eq!(ids(&before), [3, 1, 2]);
    assert_eq!(
        ids(&shown.clone().apply(paste(cue(), Place::End))),
        [1, 2, 3]
    );
}

#[test]
fn pasted_cue_keeps_a_free_number_and_takes_the_next_one_otherwise() {
    let shown = workspace(vec![audio(1), audio(2)]);
    let copy = copies(&shown, &[2]);
    let numbered =
        |workspace: &Workspace, id: u32| workspace.cue(CueId(id)).unwrap().number.clone();
    let copied = shown.clone().apply(paste(copy.clone(), Place::End));
    assert_eq!(numbered(&copied, 3), "3");
    // Cue that was cut leaves its number free, so that moving it keeps it
    let moved = shown
        .apply(delete(&[2]))
        .apply(paste(copy.clone(), Place::End));
    assert_eq!(numbered(&moved, 3), "2");
    let twice = copied.apply(paste([copy.clone(), copy].concat(), Place::End));
    assert_eq!(
        (numbered(&twice, 4), numbered(&twice, 5)),
        ("4".to_owned(), "5".to_owned())
    );
}

#[test]
fn paste_is_one_change_that_undo_takes_back_and_redo_makes_again() {
    let shown = workspace(vec![audio(1), audio(2)]);
    let cues = [copies(&shown, &[1]), copies(&shown, &[2])].concat();
    let pasted = shown.apply(paste(cues, Place::After(CueId(1))));
    let undone = pasted.apply(Command::Undo);
    assert_eq!((ids(&undone), standby(&undone)), (vec![1, 2], Some(1)));
    assert_eq!(ids(&undone.apply(Command::Redo)), [1, 3, 4, 2]);
}

#[test]
fn pasting_at_the_end_of_a_list_played_to_its_end_puts_standby_on_the_first_copy() {
    let played = workspace(vec![audio(1)]).apply(GO);
    let cues = copies(&played, &[1, 1]);
    let pasted = played.apply(paste(cues, Place::End));
    assert_eq!(standby(&pasted), Some(2));
    assert_eq!(standby(&pasted.apply(Command::Undo)), None);
}

#[test]
fn pasting_nothing_or_in_show_mode_changes_nothing() {
    let shown = workspace(vec![audio(1)]);
    let cues = copies(&shown, &[1]);
    let empty = shown.clone().apply(paste(Vec::new(), Place::End));
    assert!(empty.undoable().is_none());
    let locked = shown
        .apply(Command::ShowMode(true))
        .apply(paste(cues, Place::End));
    assert_eq!(ids(&locked), [1]);
}

#[test]
fn copy_holds_its_file_by_absolute_path_and_paste_makes_it_relative_where_it_lands() {
    let cue = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .file("sounds/rain.ogg")
        .build();
    let from = Workspace::new(vec![CueList::new("Main", vec![cue])]).directory("/shows/storm");
    let cues = copies(&from, &[1]);
    assert_eq!(cues[0].file, "/shows/storm/sounds/rain.ogg");
    let file_after = |directory: &str| {
        let into = Workspace::new(Vec::new()).directory(directory);
        let pasted = into.apply(paste(cues.clone(), Place::End));
        pasted.lists()[0].cues[0].file.clone()
    };
    assert_eq!(file_after("/shows/storm"), "sounds/rain.ogg");
    assert_eq!(file_after("/shows/storm/sounds"), "rain.ogg");
    assert_eq!(file_after("/shows/calm"), "/shows/storm/sounds/rain.ogg");
}

#[test]
fn copy_forgets_what_loading_found_and_a_cue_without_a_file_keeps_none() {
    let shown = workspace(vec![
        audio(1),
        Cue::new(CueId(2), "2", Kind::Audio, "Empty").build(),
    ]);
    let cues = copies(&shown, &[1, 2]);
    assert_eq!(cues[0].media, Media::Unloaded);
    assert_eq!(cues[1].file, "");
}

#[test]
fn pasted_cue_keeps_a_target_that_is_a_cue_of_the_show_and_loses_any_other() {
    let fade = Cue::new(CueId(2), "2", Kind::Fade, "Fade")
        .target(CueId(1))
        .build();
    let shown = workspace(vec![audio(1), fade]);
    let cue = copies(&shown, &[2]);
    let target =
        |workspace: &Workspace| workspace.lists()[0].cues.last().and_then(|cue| cue.target);
    let kept = shown.apply(paste(cue.clone(), Place::End));
    assert_eq!(target(&kept), Some(CueId(1)));
    let gone = kept.apply(delete(&[1]));
    let dropped = gone.apply(paste(cue, Place::End));
    assert_eq!(target(&dropped), None);
}

#[test]
fn a_target_among_the_cues_pasted_goes_to_their_copy_in_each_round() {
    let fade = |id, target| {
        Cue::new(CueId(id), id.to_string(), Kind::Fade, "Fade")
            .target(CueId(target))
            .build()
    };
    // First fade targets the cue after it, and the second the cue before
    let shown = workspace(vec![fade(1, 2), audio(2), fade(3, 4), audio(4)]);
    let cues = copies(&shown, &[1, 2]);
    let twice = [cues.clone(), cues].concat();
    let pasted = shown.apply(paste(twice, Place::End));
    let targets: Vec<_> = pasted.lists()[0]
        .cues
        .iter()
        .map(|cue| (cue.id.0, cue.target.map(|target| target.0)))
        .collect();
    assert_eq!(
        targets[4..],
        [(5, Some(6)), (6, None), (7, Some(8)), (8, None)]
    );
}

#[test]
fn a_thousand_cues_paste_into_a_list_of_two_thousand_with_the_next_free_numbers() {
    let shown = workspace((1..=2000).map(audio).collect());
    let cues = copies(&shown, &(1..=1000).collect::<Vec<_>>());
    // A size at which pasting that is quadratic in the list stalls the
    // interface thread, where GO must not wait; no time limit is asserted,
    // since one fails under emulation, so the set of taken numbers in
    // `Numbers` keeps this linear by its structure
    let pasted = shown.apply(paste(cues, Place::End));
    let numbers: Vec<String> = pasted
        .cues()
        .skip(2000)
        .map(|cue| cue.number.clone())
        .collect();
    assert_eq!(numbers.first().map(String::as_str), Some("2001"));
    assert_eq!(numbers.last().map(String::as_str), Some("3000"));
    assert_eq!(pasted.cues().count(), 3000);
}
