// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the wording of failures

use super::unreadable;

#[test]
fn unreadable_file_is_told_without_its_error_number() {
    // EACCES on POSIX systems, ERROR_ACCESS_DENIED on Windows
    let denied = std::io::Error::from_raw_os_error(if cfg!(windows) { 5 } else { 13 });
    assert_eq!(unreadable(denied), "permission denied");
}
