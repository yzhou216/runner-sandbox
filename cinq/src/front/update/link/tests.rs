// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where the tutorial is looked for, and of what is said when it
//! is not there

use super::*;

/// Data directories that `XDG_DATA_HOME` and `XDG_DATA_DIRS` give, with
/// `/home/alice` as the home directory
#[cfg(unix)]
fn data(home: Option<&str>, dirs: Option<&str>) -> Vec<PathBuf> {
    data_directories(
        home.map(OsString::from),
        dirs.map(OsStr::new),
        Some(PathBuf::from("/home/alice")),
    )
}

#[cfg(unix)]
#[test]
fn data_directories_default_as_the_xdg_specification_sets_out() {
    let defaults =
        ["/home/alice/.local/share", "/usr/local/share", "/usr/share"].map(PathBuf::from);
    assert_eq!(data(None, None), defaults);
    assert_eq!(data(Some(""), Some("")), defaults);
    assert_eq!(data(Some("data"), Some("share:other/share")), defaults);
}

#[cfg(unix)]
#[test]
fn data_directories_put_the_users_first_and_skip_relative_paths() {
    let found = data(Some("/data"), Some("/nix/profile/share:share:/usr/share"));
    assert_eq!(
        found,
        ["/data", "/nix/profile/share", "/usr/share"].map(PathBuf::from)
    );
}

#[test]
fn missing_tutorial_names_every_directory_searched() {
    let directories = ["a", "b", "c"].map(PathBuf::from);
    let missing = |count| not_found("tutorial.pdf", &directories[..count]);
    assert_eq!(missing(0), "Cannot find tutorial.pdf");
    assert_eq!(missing(1), "Cannot find tutorial.pdf in a");
    assert_eq!(missing(2), "Cannot find tutorial.pdf in a or b");
    assert_eq!(missing(3), "Cannot find tutorial.pdf in a, b, or c");
}
