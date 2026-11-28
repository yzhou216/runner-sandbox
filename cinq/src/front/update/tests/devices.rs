// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of listing the audio devices, whose answers come in any order

use crate::front::App;

fn listed(id: &str) -> Result<Vec<(String, String)>, String> {
    Ok(vec![(id.to_owned(), id.to_owned())])
}

#[test]
fn listing_that_ends_after_a_newer_one_does_not_replace_it() {
    let (app, first) = App::default().list_devices();
    let (app, second) = app.list_devices();
    assert_eq!((first.units(), second.units()), (1, 1));
    let newer = app.with_audio_devices(2, listed("new"));
    let after = newer.with_audio_devices(1, listed("old"));
    assert_eq!(after.audio_devices, [("new".to_owned(), "new".to_owned())]);
}

#[test]
fn listing_that_ends_before_a_newer_one_is_ignored() {
    let (app, _) = App::default().list_devices();
    let (app, _) = app.list_devices();
    let early = app.with_audio_devices(1, listed("old"));
    assert!(early.audio_devices.is_empty());
}

#[test]
fn first_listing_of_a_start_applies() {
    let app = App::default().with_audio_devices(0, listed("first"));
    assert_eq!(
        app.audio_devices,
        [("first".to_owned(), "first".to_owned())]
    );
}
