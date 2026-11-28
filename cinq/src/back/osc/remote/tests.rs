// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the commands that addresses give

use std::thread;

use super::{Reading, Remote};
use crate::back::osc::incoming::{Argument, Message};

fn reading(address: &str, arguments: Vec<Argument>) -> Reading {
    Message {
        address: address.to_owned(),
        arguments,
    }
    .reading()
}

fn command(address: &str) -> Reading {
    reading(address, Vec::new())
}

#[test]
fn addresses_give_the_commands_that_qlab_names() {
    let numbered = |make: fn(String) -> Remote| Reading::Command(make("5".to_owned()));
    assert_eq!(command("/go"), Reading::Command(Remote::Go));
    assert_eq!(command("/panic"), Reading::Command(Remote::Panic));
    assert_eq!(command("/pause"), Reading::Command(Remote::Pause));
    assert_eq!(command("/resume"), Reading::Command(Remote::Resume));
    assert_eq!(command("/cue/5/start"), numbered(Remote::Start));
    assert_eq!(command("/cue/5/stop"), numbered(Remote::Stop));
    assert_eq!(command("/cue/5/pause"), numbered(Remote::PauseCue));
    assert_eq!(command("/cue/5/resume"), numbered(Remote::ResumeCue));
    assert_eq!(command("/playhead/5"), numbered(Remote::Playhead));
}

#[test]
fn workspace_prefix_goes_by_whatever_the_id() {
    assert_eq!(
        command("/workspace/6BA7B810-9DAD/go"),
        Reading::Command(Remote::Go)
    );
    assert_eq!(
        command("/workspace/x/cue/2.5/start"),
        Reading::Command(Remote::Start("2.5".to_owned()))
    );
    assert_eq!(command("/workspace/x"), Reading::Unknown);
}

#[test]
fn cue_number_is_trimmed_and_kept_as_text() {
    assert_eq!(
        command("/cue/ 1.10 /start"),
        Reading::Command(Remote::Start("1.10".to_owned()))
    );
    assert_eq!(
        command("/cue/Q 7/stop"),
        Reading::Command(Remote::Stop("Q 7".to_owned()))
    );
}

#[test]
fn other_addresses_are_unknown() {
    for address in [
        "/",
        "/gO",
        "/go/",
        "/go/now",
        "/cue/5",
        "/cue/5/start/now",
        "/cue/start",
        "/cue/5/hardStop",
        "/playhead",
        "/stop",
        "/eos/go",
    ] {
        assert_eq!(command(address), Reading::Unknown, "{address}");
    }
}

#[test]
fn lone_zero_or_false_is_a_release_and_other_arguments_are_ignored() {
    for off in [
        Argument::Int(0),
        Argument::Float(0.0),
        Argument::Bool(false),
    ] {
        assert_eq!(reading("/go", vec![off]), Reading::Released);
    }
    for on in [Argument::Int(1), Argument::Bool(true), Argument::Other] {
        assert_eq!(reading("/go", vec![on]), Reading::Command(Remote::Go));
    }
    assert_eq!(
        reading("/go", vec![Argument::Int(0), Argument::Int(1)]),
        Reading::Command(Remote::Go)
    );
    assert_eq!(
        reading("/nothing", vec![Argument::Int(0)]),
        Reading::Unknown
    );
}

#[test]
fn repeated_workspace_prefixes_are_unknown_and_use_no_stack() {
    // The size of a datagram that fits one UDP packet, read on a thread with
    // the default stack, as the listener reads it
    let address = "/workspace/x".repeat(5400) + "/go";
    let read = thread::spawn(move || command(&address)).join();
    assert_eq!(read.unwrap(), Reading::Unknown);
    assert_eq!(command("/workspace/x/workspace/y/go"), Reading::Unknown);
}
