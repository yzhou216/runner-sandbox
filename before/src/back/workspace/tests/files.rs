// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where the files of cues are

use std::path::PathBuf;

use super::*;

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
