// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where the files of cues are

use std::{fs, path::PathBuf};

use tempfile::TempDir;

use super::{super::files::written, *};

fn playing(file: &str) -> Cue {
    Cue {
        file: file.to_owned(),
        ..audio(1)
    }
}

#[test]
fn file_paths_leave_out_surrounding_spaces() {
    let workspace = Workspace::new(Vec::new()).directory("/show");
    assert_eq!(
        workspace.path(&playing(" sounds/storm.ogg\n")),
        Some(PathBuf::from("/show/sounds/storm.ogg"))
    );
    assert_eq!(workspace.path(&playing(" \n")), None);
}

#[test]
fn chosen_sound_file_goes_by_its_path_from_the_show_when_it_is_there() {
    let workspace = Workspace::new(Vec::new()).directory("/show");
    let name = |path: &str| workspace.file_name_of(Path::new(path));
    assert_eq!(name("/show/sounds/storm.ogg"), "sounds/storm.ogg");
    assert_eq!(name("/elsewhere/storm.ogg"), "/elsewhere/storm.ogg");
    assert_eq!(name("/showcase/storm.ogg"), "/showcase/storm.ogg");
}

#[test]
fn sound_files_are_looked_for_next_to_the_file_a_cue_plays() {
    let workspace = Workspace::new(Vec::new()).directory("/show");
    let directory = |file: &str| workspace.file_directory(&playing(file));
    assert_eq!(directory("sounds/storm.ogg"), Path::new("/show/sounds"));
    assert_eq!(directory("/music/storm.ogg"), Path::new("/music"));
    assert_eq!(directory(""), Path::new("/show"));
}

#[test]
fn relative_names_join_their_components_with_slashes() {
    let name = |parts: &[&str]| written(&parts.iter().collect::<PathBuf>());
    assert_eq!(name(&["sounds", "storm.ogg"]), "sounds/storm.ogg");
    assert_eq!(name(&["..", "sounds", "storm.ogg"]), "../sounds/storm.ogg");
    assert_eq!(name(&["storm.ogg"]), "storm.ogg");
}

#[test]
fn absolute_names_stay_as_they_are() {
    assert_eq!(written(Path::new("/music/storm.ogg")), "/music/storm.ogg");
}

#[test]
fn chosen_file_goes_by_its_path_from_a_show_directory_reached_through_dots() {
    let root = TempDir::new().unwrap();
    let shows = root.path().join("shows");
    fs::create_dir_all(&shows).unwrap();
    fs::create_dir_all(root.path().join("other")).unwrap();
    fs::write(shows.join("storm.ogg"), "").unwrap();
    let workspace = Workspace::new(Vec::new()).directory(root.path().join("other/../shows"));
    assert_eq!(
        workspace.file_name_of(&shows.join("storm.ogg")),
        "storm.ogg"
    );
}

#[cfg(unix)]
#[test]
fn chosen_file_that_is_a_link_keeps_its_own_name() {
    let root = TempDir::new().unwrap();
    let shows = root.path().join("shows");
    fs::create_dir_all(&shows).unwrap();
    fs::create_dir_all(root.path().join("x")).unwrap();
    fs::write(root.path().join("storm.ogg"), "").unwrap();
    std::os::unix::fs::symlink(root.path().join("storm.ogg"), shows.join("link.ogg")).unwrap();
    let workspace = Workspace::new(Vec::new()).directory(root.path().join("x/../shows"));
    assert_eq!(workspace.file_name_of(&shows.join("link.ogg")), "link.ogg");
}
