// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the buffer of held output and of what comes out of it

use super::buffer::{Buffer, CAP};

fn written(buffer: Buffer) -> String {
    String::from_utf8_lossy(&buffer.written()).into_owned()
}

#[test]
fn bytes_come_out_in_the_order_they_were_held() {
    let buffer = Buffer::new()
        .pushed(b"first\n")
        .pushed(b"")
        .pushed(b"second\n");
    assert_eq!(written(buffer), "first\nsecond\n");
}

#[test]
fn nothing_held_comes_out_as_nothing() {
    assert_eq!(written(Buffer::new()), "");
}

#[test]
fn buffer_keeps_the_newest_bytes_and_counts_the_dropped() {
    let old = vec![b'a'; CAP - 2];
    let buffer = Buffer::new().pushed(&old).pushed(b"bcdef");
    let out = written(buffer);
    let (notice, kept) = out.split_once('\n').expect("a notice line comes first");
    assert_eq!(
        notice,
        "cinq: 3 bytes of output from while the problems showed were dropped"
    );
    assert_eq!(kept.len(), CAP);
    assert!(
        kept.starts_with('a') && kept.ends_with("abcdef"),
        "{kept:?}"
    );
}

#[test]
fn buffer_filled_to_its_cap_drops_nothing() {
    let buffer = Buffer::new().pushed(&vec![b'a'; CAP]);
    assert_eq!(written(buffer).len(), CAP);
}

#[test]
fn bytes_that_alone_exceed_the_cap_keep_their_end() {
    let mut big = vec![b'a'; 10];
    big.extend(vec![b'b'; CAP]);
    let out = written(Buffer::new().pushed(b"older").pushed(&big));
    let (notice, kept) = out.split_once('\n').expect("a notice line comes first");
    assert_eq!(
        notice,
        format!(
            "cinq: {} bytes of output from while the problems showed were dropped",
            5 + 10
        )
    );
    assert_eq!(kept, "b".repeat(CAP));
}

#[test]
fn dropped_bytes_add_up_over_many_pushes() {
    let buffer = (0..4).fold(Buffer::new(), |buffer, _| {
        buffer.pushed(&vec![b'x'; CAP / 2])
    });
    let notice = format!("cinq: {CAP} bytes of output from while the problems showed were dropped");
    assert_eq!(written(buffer).lines().next(), Some(notice.as_str()));
}

/// Tests that run in a process of their own, since they point the
/// descriptors of standard output and error at a pipe
#[cfg(unix)]
mod redirect {
    use std::{
        env,
        io::Write,
        process::{Command, Stdio},
    };

    use super::super::{hold, release, terminal, write};

    /// Variable that makes [`child`] run, in the process that the other test
    /// starts
    const CHILD: &str = "CINQ_HELD_TEST_CHILD";

    /// Body of the child process: what is written to descriptor 2 and with
    /// [`write`] between [`hold`] and [`release`] must come out only at
    /// [`release`], and what is written to [`terminal`] at once
    #[test]
    fn child() {
        if env::var_os(CHILD).is_none() {
            return;
        }
        let descriptor = |bytes: &[u8]| {
            rustix::io::write(rustix::stdio::stderr(), bytes).expect("descriptor 2 takes bytes")
        };
        let screen = |bytes: &[u8]| {
            terminal()
                .and_then(|mut terminal| terminal.write_all(bytes))
                .expect("the terminal takes bytes")
        };
        screen(b"before\n");
        hold();
        descriptor(b"descriptor\n");
        write(b"own\n");
        screen(b"screen\n");
        release();
        descriptor(b"after\n");
    }

    #[test]
    fn output_to_descriptors_waits_for_the_release() {
        let output = Command::new(env::current_exe().expect("the test binary has a path"))
            .args(["--exact", "held::tests::redirect::child", "--nocapture"])
            .env(CHILD, "1")
            .stdin(Stdio::null())
            .output()
            .expect("the test binary runs");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{error}");
        // The thread of the redirect hands over descriptor 2 apart from the
        // writes of cinq, so those two come in either order
        assert!(
            matches!(
                &*error,
                "before\nscreen\ndescriptor\nown\nafter\n"
                    | "before\nscreen\nown\ndescriptor\nafter\n"
            ),
            "{error:?}"
        );
    }
}
