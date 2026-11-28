// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of opening the socket again while it fails

use std::{cell::Cell, io, iter, net::UdpSocket, sync::mpsc, time::Duration};

use iced::futures::channel::mpsc as reporting;

use super::bound;
use crate::back::light::Universes;

const SHORT: Duration = Duration::from_millis(20);

fn fails_then_binds(failures: usize) -> impl Fn() -> io::Result<UdpSocket> {
    let tries = Cell::new(0);
    move || {
        tries.set(tries.get() + 1);
        if tries.get() <= failures {
            Err(io::Error::other("no free descriptor"))
        } else {
            UdpSocket::bind(("127.0.0.1", 0))
        }
    }
}

#[test]
fn socket_that_fails_to_open_is_tried_again_and_the_failure_and_the_success_are_told() {
    let (sender, levels) = mpsc::channel();
    let (reporter, mut reports) = reporting::unbounded();
    let universes = Universes::from([(1, [7; 512])]);
    sender.send(universes.clone()).unwrap();
    let (_, latest) = bound(&levels, &reporter, fails_then_binds(2), SHORT)
        .expect("the socket opens at the third try");
    assert_eq!(latest, universes, "the levels that came meanwhile are kept");
    let told: Vec<_> = iter::from_fn(|| reports.try_recv().ok()).collect();
    assert_eq!(
        told,
        [Some("not sent: no free descriptor".to_owned()), None],
        "told once at the failure and once at the success"
    );
}

#[test]
fn socket_that_opens_at_once_tells_nothing() {
    let (_sender, levels) = mpsc::channel();
    let (reporter, mut reports) = reporting::unbounded();
    assert!(bound(&levels, &reporter, fails_then_binds(0), SHORT).is_some());
    assert!(reports.try_recv().is_err());
}

#[test]
fn thread_that_cannot_open_its_socket_ends_when_the_sender_is_dropped() {
    let (sender, levels) = mpsc::channel();
    let (reporter, _reports) = reporting::unbounded();
    drop(sender);
    assert!(bound(&levels, &reporter, fails_then_binds(usize::MAX), SHORT).is_none());
}
