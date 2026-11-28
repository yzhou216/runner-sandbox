// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of when a warning comes out

use std::io;

use super::{LIMIT, Warnings, words};

#[test]
fn warning_comes_out_once_after_cinq() {
    let (warnings, first) = Warnings::default().warned("a.mp4: skipped its tags");
    assert_eq!(first, "cinq: a.mp4: skipped its tags\n");
    let (warnings, again) = warnings.warned("a.mp4: skipped its tags");
    assert_eq!(again, "");
    let (_, other) = warnings.warned("b.mp4: skipped its tags");
    assert_eq!(other, "cinq: b.mp4: skipped its tags\n");
}

#[test]
fn warnings_end_with_one_line_after_the_limit() {
    let warnings = (0..LIMIT).fold(Warnings::default(), |warnings, number| {
        let (warnings, line) = warnings.warned(&format!("warning {number}"));
        assert_eq!(line, format!("cinq: warning {number}\n"));
        warnings
    });
    let (warnings, repeated) = warnings.warned("warning 0");
    assert_eq!(repeated, "");
    let (warnings, last) = warnings.warned("one more");
    assert_eq!(last, "cinq: further warnings go unreported\n");
    let (warnings, silent) = warnings.warned("yet another");
    assert_eq!(silent, "");
    assert_eq!(warnings.given.len(), LIMIT);
}

#[test]
fn error_from_the_system_reads_as_its_kind() {
    let error = io::Error::from_raw_os_error(2);
    assert_eq!(words(&error), error.kind().to_string());
    // Numbers of the errors differ between systems, as that of a refused
    // connection is 111 on GNU/Linux and 61 on Darwin
    #[cfg(unix)]
    {
        use rustix::io::Errno;
        let word = |errno: Errno| words(&io::Error::from_raw_os_error(errno.raw_os_error()));
        assert_eq!(word(Errno::ACCESS), "permission denied");
        assert_eq!(word(Errno::CONNREFUSED), "connection refused");
    }
}

#[test]
fn error_of_cinq_keeps_its_own_words() {
    assert_eq!(
        words(&io::Error::new(io::ErrorKind::TimedOut, "dropped late")),
        "dropped late"
    );
    assert_eq!(
        words(&io::Error::other("no IPv4 address")),
        "no IPv4 address"
    );
}

#[cfg(unix)]
#[test]
fn error_of_the_system_without_a_kind_reads_as_its_message_without_the_number() {
    use rustix::io::Errno;
    // Neither of these has a kind of its own, and neither number is the
    // same on every system, so that only the shape of the words is checked
    let errors: Vec<io::Error> = [Errno::NXIO, Errno::AFNOSUPPORT]
        .iter()
        .map(|errno| io::Error::from_raw_os_error(errno.raw_os_error()))
        .filter(|error| error.kind().to_string() == "uncategorized error")
        .collect();
    // A later standard library may give both a kind, which would leave
    // nothing to check and the rule untested
    assert!(
        !errors.is_empty(),
        "no error without a kind is left to check"
    );
    errors.iter().for_each(|error| {
        let text = words(error);
        assert!(!text.contains("os error"), "{text}");
        assert_ne!(text, "uncategorized error");
        assert!(
            text.starts_with(|letter: char| letter.is_lowercase()),
            "{text}"
        );
    });
}
