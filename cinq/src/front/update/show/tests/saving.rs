// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of saving shows, which leaves unsaved only the changes made since,
//! in a file with the ending of a show and the permissions it had

use super::*;

#[test]
fn show_changed_and_changed_back_has_nothing_to_save() {
    let app = storm();
    assert!(!app.show.unsaved());
    let changed = renamed(app, "Cue");
    assert!(changed.show.unsaved());
    assert!(!changed.send(Command::Undo).show.unsaved());
}

#[test]
fn saving_leaves_unsaved_only_the_changes_made_since() {
    let new = App::default()
        .opened(found(None, None, None))
        .send(Command::Add {
            list: 0,
            kind: Kind::Memo,
            at: Place::End,
        });
    let file = "/shows/new.cinq.toml";
    let save = saving(&new, file);
    let after = renamed(new, "Later").saved(save);
    assert_eq!(after.show.name(), "new.cinq.toml");
    assert!(after.show.unsaved());
    let again = saving(&after, file);
    assert!(!after.saved(again).show.unsaved());
}

#[test]
fn failed_saving_leaves_the_changes_unsaved() {
    let changed = renamed(storm(), "Cue");
    let failed = Saved {
        result: Err("disk full".to_owned()),
        ..saving(&changed, STORM)
    };
    let app = changed.saved(failed);
    assert!(app.show.unsaved());
    assert_eq!(
        app.file_problem.as_deref(),
        Some("Cannot save /shows/storm.cinq.toml: disk full")
    );
}

#[test]
fn save_that_ends_after_another_show_opens_leaves_that_show_alone() {
    let changed = renamed(storm(), "Cue");
    let save = saving(&changed, STORM);
    let calm = changed.opened(found(Some(CALM), Some(SHOW), None));
    let saved = calm.show.saved.clone();
    let after = calm.saved(save);
    assert_eq!(after.show.name(), "calm.cinq.toml");
    assert_eq!(after.show.saved, saved);
    assert!(!after.show.unsaved());
}

#[test]
fn chosen_file_gets_the_ending_of_a_show_unless_it_is_toml() {
    assert_eq!(with_extension(Path::new("/shows/storm")), Path::new(STORM));
    assert_eq!(
        with_extension(Path::new("/shows/storm.toml")),
        Path::new("/shows/storm.toml")
    );
    assert_eq!(
        with_extension(Path::new("/shows/storm.v2")),
        Path::new("/shows/storm.v2.cinq.toml")
    );
}

#[cfg(unix)]
#[test]
fn saving_keeps_the_permissions_of_the_show_and_the_link_to_it() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let directory = temporary();
    let show = directory.path().join("storm.cinq.toml");
    fs::write(&show, SHOW).expect("the show can be written");
    fs::set_permissions(&show, fs::Permissions::from_mode(0o640))
        .expect("the permissions can be set");
    let link = directory.path().join("link.cinq.toml");
    symlink(&show, &link).expect("the link can be made");
    write(&link, "changed", true).expect("the show can be saved");
    assert_eq!(fs::read_to_string(&show).ok().as_deref(), Some("changed"));
    assert!(fs::symlink_metadata(&link).is_ok_and(|metadata| metadata.file_type().is_symlink()));
    let mode = fs::metadata(&show).map(|metadata| metadata.permissions().mode() & 0o777);
    assert_eq!(mode.ok(), Some(0o640));
}
