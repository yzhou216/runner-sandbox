// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what the status line says of listening and of idle commands

use super::*;

/// Returns the configuration that listens for OSC at `address`
fn listening(address: &str) -> Config {
    let mut config = Config::default();
    config.osc_listen = Some(address.parse().unwrap());
    config
}

/// Returns the state that listens for OSC at `address`
fn listening_at(address: &str) -> App {
    App {
        config: listening(address),
        ..App::default()
    }
}

/// Returns the message that tells that the configuration listens at `address`
fn reconfigured(address: &str) -> Message {
    Message::Reconfigured(Box::new(listening(address)))
}

#[test]
fn failure_to_listen_shows_until_the_address_changes_or_listening_starts() {
    let failed = || {
        let app = listening_at("127.0.0.1:53000");
        after(app, [Message::OscFailed("OSC: cannot listen".to_owned())])
    };
    assert!(failed().problems.osc.is_some());
    let listening = after(failed(), [Message::OscListening]);
    assert!(listening.problems.osc.is_none());
    let same = after(failed(), [reconfigured("127.0.0.1:53000")]);
    assert!(same.problems.osc.is_some());
    let moved = after(same, [reconfigured("127.0.0.1:53001")]);
    assert!(moved.problems.osc.is_none());
}

#[test]
fn command_that_did_nothing_shows_as_one_line_until_a_command_acts() {
    let first = remote(playable(None), Remote::Start("99".into()));
    assert_eq!(
        first.problems.osc_ignored.as_deref(),
        Some("OSC from 192.168.1.5: no cue is numbered \"99\"")
    );
    let second = remote(first, Remote::Stop("98".into()));
    assert_eq!(
        second.problems.osc_ignored.as_deref(),
        Some("OSC from 192.168.1.5: no cue is numbered \"98\""),
        "the latest replaces the line"
    );
    let heard = after(
        second,
        [Message::OscIgnored("OSC: not an OSC packet".into())],
    );
    assert_eq!(
        heard.problems.osc_ignored.as_deref(),
        Some("OSC: not an OSC packet")
    );
    assert!(remote(heard, Remote::Pause).problems.osc_ignored.is_none());
}

#[test]
fn line_of_a_command_that_did_nothing_goes_with_the_address() {
    let told = after(
        listening_at("127.0.0.1:53000"),
        [Message::OscIgnored("OSC: not an OSC packet".into())],
    );
    let moved = after(told, [reconfigured("127.0.0.1:53001")]);
    assert!(moved.problems.osc_ignored.is_none());
}

#[test]
fn number_that_no_cue_has_is_cut_short_in_the_line_that_tells_so() {
    let long = "9".repeat(100);
    let app = remote(playable(None), Remote::Start(long.clone()));
    assert_eq!(
        app.problems.osc_ignored,
        Some(format!(
            "OSC from 192.168.1.5: no cue is numbered {:?}",
            &long[..64]
        ))
    );
}
