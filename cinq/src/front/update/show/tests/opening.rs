// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of opening shows, which brings back the changes kept for them,
//! and of reading them from their files

use super::*;

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
    assert_eq!(app.told, None);
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
    assert!(app.file_problem.is_some());
}

#[test]
fn opening_another_show_tells_where_the_unsaved_changes_went() {
    let told = "Kept the unsaved changes to storm.cinq.toml | Control+o: open";
    let app = renamed(storm(), "Cue").opened(found(Some(CALM), Some(SHOW), None));
    assert_eq!(app.told.as_deref(), Some(told));
    // Also when the show opened brings back unsaved changes of its own
    let kept = SHOW.replace("Note", "Calm");
    let app = renamed(storm(), "Cue").opened(found(Some(CALM), Some(SHOW), Some(&kept)));
    assert_eq!(app.told.as_deref(), Some(told));
    // Without the key for a new show, whose changes only starting cinq
    // without a show brings back
    let untitled = App::default().opened(found(None, None, Some(SHOW)));
    let app = untitled.opened(found(Some(CALM), Some(SHOW), None));
    assert_eq!(
        app.told.as_deref(),
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
    assert!(app.file_problem.is_some() && app.keep_problem.is_some());
    let opened = app.opened(found(Some(CALM), Some(SHOW), None));
    assert!(opened.file_problem.is_none() && opened.keep_problem.is_none());
    let app = failed();
    let save = saving(&app, STORM);
    assert!(app.saved(save).keep_problem.is_none());
}

#[test]
fn failure_to_keep_takes_back_the_note_that_changes_were_kept() {
    let app = renamed(storm(), "Cue").opened(found(Some(CALM), Some(SHOW), None));
    assert!(app.told.is_some());
    assert!(app.with_keep_problem(None).told.is_some());
    let app = renamed(storm(), "Cue").opened(found(Some(CALM), Some(SHOW), None));
    assert_eq!(app.with_keep_problem(Some("late".to_owned())).told, None);
}
