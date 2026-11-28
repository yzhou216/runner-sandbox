// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of opening shows, which brings back the changes kept for them,
//! and of reading them from their files

use super::*;
use crate::front::Message;

#[test]
fn opening_brings_back_kept_changes_as_one_change_that_undo_takes_back() {
    let kept = SHOW.replace("Note", "Changed");
    let app = App::default().opened(found(Some(STORM), Some(SHOW), Some(&kept)));
    assert!(app.show.unsaved());
    let name = app.workspace.cue(CueId(1)).map(|cue| cue.name.as_str());
    assert_eq!(name, Some("Changed"));
    assert!(!app.send(Command::Undo).show.unsaved());
}

#[test]
fn opening_again_the_show_on_screen_brings_back_its_changes_on_screen() {
    let older = SHOW.replace("Note", "Older");
    let app = renamed(storm(), "Newer").opened(found(Some(STORM), Some(SHOW), Some(&older)));
    let name = app.workspace.cue(CueId(1)).map(|cue| cue.name.as_str());
    assert_eq!(name, Some("Newer"));
    // Nothing to tell of changes that stay on screen
    assert_eq!(app.notices.told, None);
    assert!(!app.send(Command::Undo).show.unsaved());
}

#[test]
fn new_show_brings_back_the_kept_changes_of_the_untitled_show() {
    let kept = SHOW.replace("Note", "Changed");
    let app = App::default().opened(found(None, None, Some(&kept)));
    let name = app.workspace.cue(CueId(1)).map(|cue| cue.name.as_str());
    assert_eq!(name, Some("Changed"));
    // Selection is the standby of the cues that came back
    assert_eq!(app.selected, Some(CueId(1)));
}

#[test]
fn opening_a_file_that_is_not_there_starts_a_new_show_that_saving_puts_there() {
    let app = App::default().opened(found(Some("/shows/new.cinq.toml"), None, None));
    assert_eq!(app.show.name(), "new.cinq.toml");
    assert_eq!(app.workspace.lists().len(), 1);
    assert!(!app.show.unsaved());
}

#[test]
fn file_that_is_not_a_show_leaves_the_show_on_screen() {
    let app = storm().opened(found(Some("/shows/broken.cinq.toml"), Some("[[list"), None));
    assert_eq!(app.show.name(), "storm.cinq.toml");
    assert!(app.problems.file.is_some());
}

#[test]
fn file_that_is_not_a_show_is_told_by_its_name_and_where_it_goes_wrong_in_one_line() {
    let text = "[[list]]\nname = \"Main\"\n[[list.cue]]\nname = \"Knock\nid = 1\n";
    let app = App::default().opened(found(Some("/shows/big.cinq.toml"), Some(text), None));
    assert!(app.problems.open_failed);
    let problem = app.problems.file.as_deref().unwrap_or_default();
    assert!(
        problem.starts_with("Cannot open big.cinq.toml: line 4, column 14: "),
        "{problem}"
    );
    assert_eq!(problem.lines().count(), 1, "{problem}");
    let opened = app.opened(found(Some(STORM), Some(SHOW), None));
    assert!(!opened.problems.open_failed && opened.problems.file.is_none());
}

#[test]
fn opening_another_show_tells_where_the_unsaved_changes_went() {
    let told = "Kept the unsaved changes to storm.cinq.toml | Control+l: open";
    let app = renamed(storm(), "Cue").opened(found(Some(CALM), Some(SHOW), None));
    assert_eq!(app.notices.told.as_deref(), Some(told));
    // Also when the show opened brings back unsaved changes of its own
    let kept = SHOW.replace("Note", "Calm");
    let app = renamed(storm(), "Cue").opened(found(Some(CALM), Some(SHOW), Some(&kept)));
    assert_eq!(app.notices.told.as_deref(), Some(told));
    // Without the key for a new show, whose changes only starting cinq
    // without a show brings back
    let untitled = App::default().opened(found(None, None, Some(SHOW)));
    let app = untitled.opened(found(Some(CALM), Some(SHOW), None));
    assert_eq!(
        app.notices.told.as_deref(),
        Some("Kept the unsaved changes to untitled")
    );
}

#[test]
fn show_read_back_holds_what_was_written_along_with_its_kept_changes() {
    let directory = temporary();
    let file = directory.path().join("storm.cinq.toml");
    let unsaved_directory = directory.path().join("unsaved");
    write(&file, SHOW, true).expect("the show can be written");
    write(&kept_file(&unsaved_directory, Some(&file)), "kept", false).expect("changes can be kept");
    let keeper = Keeper::new(Some(unsaved_directory));
    let read = Read::new(Some(file), keeper.reader().as_ref());
    assert_eq!(read.text, Ok(Some(SHOW.to_owned())));
    assert_eq!(read.kept, Ok(Some("kept".to_owned())));
}

#[test]
fn failure_to_keep_outlasts_a_problem_of_opening_until_a_show_opens_or_saves() {
    let failed = || storm().with_keep_problem(Some("Cannot keep the unsaved changes".to_owned()));
    let broken = found(Some("/shows/broken.cinq.toml"), Some("[[list"), None);
    let app = failed().opened(broken);
    assert!(app.problems.file.is_some() && app.problems.keep.is_some());
    let opened = app.opened(found(Some(CALM), Some(SHOW), None));
    assert!(opened.problems.file.is_none() && opened.problems.keep.is_none());
    let app = failed();
    let save = saving(&app, STORM);
    assert!(app.saved(save).problems.keep.is_none());
}

#[test]
fn failure_to_keep_takes_back_the_note_that_changes_were_kept() {
    let app = renamed(storm(), "Cue").opened(found(Some(CALM), Some(SHOW), None));
    assert!(app.notices.told.is_some());
    assert!(app.with_keep_problem(None).notices.told.is_some());
    let app = renamed(storm(), "Cue").opened(found(Some(CALM), Some(SHOW), None));
    assert_eq!(
        app.with_keep_problem(Some("late".to_owned())).notices.told,
        None
    );
}

#[test]
fn opening_another_show_clears_the_places_to_go_back_to() {
    let lists = SHOW.replace("Main", "Main\"\n\n[[list]]\nname = \"Preshow");
    let app = App::default()
        .opened(found(Some(STORM), Some(&lists), None))
        .next(Message::SwitchList(1))
        .0;
    assert!(app.can_go_back());
    let app = app.opened(found(Some(CALM), Some(SHOW), None));
    assert!(!app.can_go_back() && !app.can_go_forward());
}

#[test]
fn opening_another_show_resets_the_rows_of_a_half_page_that_a_count_set() {
    let app = App {
        scroll: Some(5),
        ..App::default()
    };
    let app = app.opened(found(Some(CALM), Some(SHOW), None));
    assert_eq!(app.scroll, None);
}

/// Show file of an audio cue and a fade cue that targets it
const FADING: &str = "[[list]]\nname = \"Main\"\n\n[[list.cue]]\nid = 1\nnumber = \"1\"\nkind = \"audio\"\nname = \"Rain\"\nfile = \"rain.ogg\"\n\n[[list.cue]]\nid = 2\nnumber = \"2\"\nkind = \"fade\"\nname = \"Fade\"\ntarget = 1\n";

#[test]
fn clipboard_outlasts_the_show_but_its_cues_lose_the_targets_of_that_show() {
    let target = |app: &App| {
        app.workspace.lists()[0]
            .cues
            .last()
            .and_then(|cue| cue.target)
    };
    let copied = App::default()
        .opened(found(Some(STORM), Some(FADING), None))
        .with_selection(CueId(2))
        .with_copied(1);
    let here = copied.with_pasted(Place::After, 1);
    assert_eq!(here.workspace.lists()[0].cues.len(), 3);
    let at_end = here.with_pasted(|_| Place::End, 1);
    assert_eq!(target(&at_end), Some(CueId(1)));
    // Cue 1 of the other show is not the cue 1 that the fade targeted
    let elsewhere = at_end
        .opened(found(Some(CALM), Some(FADING), None))
        .with_pasted(|_| Place::End, 1);
    assert_eq!(elsewhere.workspace.lists()[0].cues.len(), 3);
    assert_eq!(target(&elsewhere), None);
}

#[test]
fn a_fade_pasted_into_another_show_with_its_target_targets_the_copy() {
    let copied = App::default()
        .opened(found(Some(STORM), Some(FADING), None))
        .with_selection(CueId(1))
        .with_copied(2);
    let elsewhere = copied
        .opened(found(Some(CALM), Some(FADING), None))
        .with_pasted(|_| Place::End, 1);
    assert_eq!(
        crate::back::targets(&elsewhere.workspace),
        [(1, None), (2, Some(1)), (3, None), (4, Some(3))]
    );
}

#[test]
fn opening_a_show_closes_the_search_line() {
    let (searching, _) = storm().searching(true);
    assert!(searching.search.is_some());
    let app = searching.opened(found(Some(CALM), Some(SHOW), None));
    assert!(app.search.is_none());
}

#[test]
fn opening_a_show_over_the_outputs_selects_one_of_its_outputs() {
    let app = storm().handled(Message::ShowOutputs).0;
    let added = (0..2).fold(app, |app, _| app.handled(Message::Add(Place::After, 1)).0);
    let opened = added.opened(found(Some(CALM), Some(SHOW), None));
    assert!(opened.outputs_shown);
    assert!(opened.output_row().is_some());
}

#[test]
fn directory_in_place_of_a_show_is_told_without_an_error_number() {
    // Path of its own, since that of temp_dir on Windows ends in a separator,
    // which makes it read as missing
    let directory = tempfile::tempdir().unwrap();
    let reason = crate::front::files::read(directory.path()).expect_err("a directory is no show");
    assert!(!reason.contains("os error"), "{reason}");
}

#[test]
fn opening_a_show_leaves_hint_mode_since_its_labels_stand_for_the_old_show() {
    let (hinting, _) = storm().hinting();
    assert!(hinting.hints.is_some());
    let opened = hinting.opened(found(Some(CALM), Some(SHOW), None));
    assert!(opened.hints.is_none());
}
