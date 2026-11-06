// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keeper of unsaved changes, which keeps them in the state
//! directory until they are gone

use super::*;

#[test]
fn keeper_keeps_unsaved_changes_until_they_are_gone() {
    let (mut keeper, directory) = keeper();
    let kept = kept_file(directory.path(), Some(Path::new(STORM)));
    let app = storm();
    assert_eq!(keeper.keep(&app.show), None);
    keeper.settled();
    assert!(!kept.exists());
    let changed = renamed(app, "Cue");
    assert_eq!(keeper.keep(&changed.show), None);
    keeper.settled();
    assert_eq!(
        fs::read_to_string(&kept).ok(),
        Some(changed.show.text.clone())
    );
    assert_eq!(keeper.keep(&changed.send(Command::Undo).show), None);
    keeper.settled();
    assert!(!kept.exists());
}

#[test]
fn keeper_leaves_the_kept_changes_of_a_show_that_another_takes_the_place_of() {
    let (mut keeper, directory) = keeper();
    let kept = kept_file(directory.path(), Some(Path::new(STORM)));
    let changed = renamed(storm(), "Cue");
    keeper.keep(&changed.show);
    let other = changed.opened(found(Some(CALM), Some(SHOW), None));
    assert_eq!(keeper.keep(&other.show), None);
    keeper.settled();
    assert!(kept.exists());
}

#[test]
fn kept_changes_move_along_with_their_show() {
    let (mut keeper, directory) = keeper();
    let kept = kept_file(directory.path(), Some(Path::new(STORM)));
    let moved = "/elsewhere/storm.cinq.toml";
    let changed = renamed(storm(), "Cue");
    keeper.keep(&changed.show);
    let save = saving(&changed, moved);
    let saved = renamed(changed, "Later").saved(save);
    assert_eq!(keeper.keep(&saved.show), None);
    keeper.settled();
    assert!(!kept.exists());
    assert!(kept_file(directory.path(), Some(Path::new(moved))).exists());
}

#[test]
fn show_named_through_a_parent_directory_keeps_its_changes_as_by_its_plain_path() {
    let kept = |file: &str| kept_file(Path::new("/unsaved"), Some(Path::new(file)));
    assert_eq!(kept("/shows/old/../storm.cinq.toml"), kept(STORM));
}

#[test]
fn keeper_tells_at_its_next_turn_why_a_change_failed() {
    let directory = TempDir::new().expect("a temporary directory can be made");
    // File where the directory of unsaved changes would go
    let blocked = directory.path().join("unsaved");
    fs::write(&blocked, "").expect("the file can be written");
    let mut keeper = Keeper::new(Some(blocked));
    let changed = renamed(storm(), "Cue");
    assert_eq!(keeper.keep(&changed.show), None);
    keeper.settled();
    let failed = keeper.keep(&changed.show);
    assert!(
        failed.as_deref().is_some_and(
            |why| why.starts_with("Cannot keep the unsaved changes to storm.cinq.toml: ")
        ),
        "{failed:?}"
    );
}

#[test]
fn keeper_makes_every_change_sent_to_it_before_it_goes() {
    let (mut keeper, directory) = keeper();
    let kept = kept_file(directory.path(), Some(Path::new(STORM)));
    let changed = renamed(storm(), "Cue");
    keeper.keep(&changed.show);
    drop(keeper);
    assert_eq!(fs::read_to_string(&kept).ok(), Some(changed.show.text));
}
