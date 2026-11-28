// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the collapsed groups of a saved show, which opening keeps
//! collapsed and a cancelled search leaves as they were

use super::*;
use crate::{
    back::{Cue, fresh},
    front::{Message, message::Fold},
};

/// Returns the text of a show of a collapsed group of two cues, with a cue
/// below it
fn closed() -> String {
    let cue = |id: u32, depth: u8, kind: Kind, name: &str| Cue {
        depth,
        collapsed: kind == Kind::Group,
        ..Cue::new(CueId(id), id.to_string(), kind, name).build()
    };
    fresh(vec![
        cue(1, 0, Kind::Group, "Storm"),
        cue(2, 1, Kind::Memo, "Thunder"),
        cue(3, 1, Kind::Memo, "Rain"),
        cue(4, 0, Kind::Memo, "Dawn"),
    ])
    .show(Some(Path::new("/shows")))
}

fn opened_closed() -> App {
    App::default().opened(found(Some(STORM), Some(&closed()), None))
}

#[test]
fn a_show_opens_with_its_groups_collapsed_and_saved() {
    let (app, _) = App::default().handled(Message::Opened(Box::new(found(
        Some(STORM),
        Some(&closed()),
        None,
    ))));
    assert_eq!(app.selected, Some(CueId(1)));
    assert!(app.workspace.cue(CueId(1)).is_some_and(|cue| cue.collapsed));
    assert!(!app.show.unsaved());
}

/// Returns the opened show with a search for rain typed, which opens the group
fn searching_rain() -> App {
    let (app, _) = opened_closed().handled(Message::Search { backward: false });
    app.handled(Message::Searching("rain".to_owned())).0
}

#[test]
fn a_cancelled_search_leaves_the_show_as_saved() {
    let found = searching_rain();
    assert_eq!(found.selected, Some(CueId(3)));
    assert!(found.show.unsaved());
    let (cancelled, _) = found.handled(Message::Cancel);
    assert_eq!(cancelled.selected, Some(CueId(1)));
    assert!(!cancelled.show.unsaved());
}

#[test]
fn an_accepted_search_leaves_the_groups_of_its_match_open_and_unsaved() {
    let (accepted, _) = searching_rain().handled(Message::SearchAccepted);
    assert!(accepted.show.unsaved());
    let folded = accepted.handled(Message::Fold(Fold::CloseAll)).0;
    assert!(!folded.show.unsaved());
}
