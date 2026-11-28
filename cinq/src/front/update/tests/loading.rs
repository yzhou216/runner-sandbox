// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of loading the files of a show, and of the window drawn before a
//! show has opened

use super::show;
use crate::{
    back::{Load, load_files},
    front::App,
};

#[test]
fn one_load_runs_at_a_time_until_what_it_found_comes_back() {
    let app = show(None);
    let found = load_files(Load::all(&app.workspace, app.workspace.unloaded()));
    let (loading, first) = app.load();
    let (still, again) = loading.load();
    assert_eq!(first.units(), 1);
    assert_eq!(again.units(), 0);
    assert!(still.loading);
    let show = still.show.id();
    let done = still.loaded(show, found);
    assert!(!done.loading);
    assert_eq!(done.workspace.unloaded().count(), 0);
}

#[test]
fn window_draws_before_a_show_has_opened() {
    let _ = crate::front::view::view(&App::default());
}
