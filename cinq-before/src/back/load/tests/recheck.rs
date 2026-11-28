// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of looking again at the files that cues have read, which reads only
//! those whose size or modification time has changed, or that have gone

use std::fs;

use tempfile::TempDir;

use super::{findings, playing};
use crate::back::{
    CueId, Kind, Media, Workspace,
    audio::tests::silence,
    failure::unplayable,
    load::{Load, load_files},
    loaded,
    workspace::tests::fresh,
};

/// Returns a directory holding `good.wav`, which plays for a second, and
/// `fake.wav`, which holds text that no decoder reads
pub(super) fn directory() -> TempDir {
    let directory = TempDir::new().expect("a temporary directory can be made");
    fs::write(directory.path().join("good.wav"), silence(1.0)).expect("the file can be written");
    fs::write(directory.path().join("fake.wav"), "not a sound").expect("the file can be written");
    directory
}

/// Returns the workspace of an audio cue for each of `files`, as loading has
/// left it
pub(super) fn read(directory: &TempDir, files: &[&str]) -> Workspace {
    let cues = files
        .iter()
        .zip(1..)
        .map(|(file, id)| playing(id, Kind::Audio, file))
        .collect();
    let workspace = fresh(cues).directory(directory.path());
    load_files(Load::all(&workspace, workspace.unloaded()))
        .into_iter()
        .fold(workspace, Workspace::apply)
}

/// Returns what a look at the files that have been read finds that has
/// changed, with the number of the cue that each finding belongs to
pub(super) fn looked(workspace: &Workspace) -> Vec<(u32, Media)> {
    findings(load_files(Load::all(workspace, workspace.to_recheck())))
}

/// Returns the workspace once a look at its files has been applied, as the
/// interface does
pub(super) fn looked_at(workspace: Workspace) -> Workspace {
    looked(&workspace)
        .into_iter()
        .map(|(cue, media)| loaded(cue, "good.wav", media))
        .fold(workspace, Workspace::apply)
}

fn not_found() -> Media {
    Media::Failed("file not found".to_owned())
}

#[test]
fn files_that_stay_as_they_were_are_not_read_again_whether_they_play_or_not() {
    let directory = directory();
    let workspace = read(&directory, &["good.wav", "fake.wav"]);
    assert!(matches!(
        workspace.cue(CueId(2)).map(|cue| &cue.media),
        Some(Media::Failed(_))
    ));
    assert_eq!(looked(&workspace), []);
}

#[test]
fn file_that_changes_is_read_again() {
    let directory = directory();
    let workspace = read(&directory, &["good.wav"]);
    fs::write(directory.path().join("good.wav"), silence(2.0)).expect("the file can be written");
    let found = looked(&workspace);
    assert!(
        matches!(found[..], [(1, Media::Loaded { duration, .. })] if (duration - 2.0).abs() < 0.01),
        "{found:?}"
    );
}

#[test]
fn file_that_goes_is_found_gone_at_every_look_and_read_when_it_comes_back() {
    let directory = directory();
    let workspace = read(&directory, &["good.wav"]);
    let path = directory.path().join("good.wav");
    let kept = fs::read(&path).expect("the file can be read");
    fs::remove_file(&path).expect("the file can be removed");
    assert_eq!(looked(&workspace), [(1, not_found())]);
    let gone = looked_at(workspace);
    assert_eq!(looked(&gone), [(1, not_found())]);
    fs::write(&path, kept).expect("the file can be written");
    assert!(matches!(looked(&gone)[..], [(1, Media::Loaded { .. })]));
}

#[test]
fn failure_that_playing_found_stays_until_the_file_changes() {
    let directory = directory();
    let workspace = read(&directory, &["good.wav"]);
    let cue = workspace.cue(CueId(1)).expect("the cue is there");
    let failed = workspace.clone().apply(unplayable(
        &workspace,
        cue,
        "its picture is damaged".to_owned(),
    ));
    assert_eq!(
        failed.cue(CueId(1)).map(|cue| &cue.media),
        Some(&Media::Failed("its picture is damaged".to_owned()))
    );
    assert_eq!(looked(&failed), []);
    fs::write(directory.path().join("good.wav"), silence(2.0)).expect("the file can be written");
    assert!(matches!(looked(&failed)[..], [(1, Media::Loaded { .. })]));
}

#[cfg(unix)]
#[test]
fn file_that_cannot_be_opened_is_read_again_until_it_can_be() {
    use std::os::unix::fs::PermissionsExt;

    let directory = directory();
    let path = directory.path().join("good.wav");
    let set = |mode| fs::set_permissions(&path, fs::Permissions::from_mode(mode));
    set(0o000).expect("the permissions can be set");
    // A user that may read any file opens this one, and the look would have
    // nothing to tell
    if fs::File::open(&path).is_ok() {
        return;
    }
    let workspace = read(&directory, &["good.wav"]);
    assert!(matches!(looked(&workspace)[..], [(1, Media::Failed(_))]));
    set(0o644).expect("the permissions can be set");
    assert!(matches!(
        looked(&workspace)[..],
        [(1, Media::Loaded { .. })]
    ));
}
