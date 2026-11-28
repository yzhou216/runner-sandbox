// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what an edit tells of the cues that it stopped

use super::{after, on_clipboard, playing, show, typed_into, typed_setting};
use crate::{
    back::CueId,
    front::{App, Message, Setting, app::Held},
};

const CUE: CueId = CueId(1);

#[test]
fn a_cut_of_a_cue_that_plays_says_that_it_stopped() {
    let cut = after(playing(), [Message::Select(CUE), Message::Cut(1)]);
    assert!(cut.workspace.active_of(CUE).is_none());
    assert_eq!(on_clipboard(&cut), Some(("Cut cue 1, which stopped", true)));
    let several = after(playing(), [Message::Select(CUE), Message::Cut(3)]);
    assert_eq!(
        on_clipboard(&several),
        Some(("Cut 3 cues, 1 of which stopped", true))
    );
}

#[test]
fn a_cut_of_a_cue_that_is_silent_says_nothing_of_it() {
    let silent = after(show(Some(CUE)), [Message::Cut(1)]);
    assert_eq!(on_clipboard(&silent), Some(("Cut cue 1", true)));
    assert!(silent.notices.stopped.is_empty());
}

#[test]
fn a_delete_remembers_the_cue_that_it_stopped_until_undo() {
    let deleted = after(playing(), [Message::Select(CUE), Message::Delete(1)]);
    assert_eq!(deleted.notices.stopped, [CUE]);
    let undone = after(deleted, [Message::Undo(1)]);
    assert!(undone.notices.stopped.is_empty());
    assert!(undone.workspace.active_of(CUE).is_none());
}

/// Returns the state after `text` goes into the File field of the cue, a key
/// at a time
fn typed_file(app: App, text: &str) -> App {
    typed_into(app, text, |prefix| {
        Message::SettingTyped(Setting::File(CUE), prefix)
    })
}

#[test]
fn a_path_typed_into_the_file_of_a_cue_that_plays_leaves_it_playing_until_the_field_is_left() {
    let typing = typed_file(playing(), "bell.ogg");
    assert!(typing.workspace.active_of(CUE).is_some());
    assert_eq!(typing.workspace.cue(CUE).unwrap().file, "sound.ogg");
    assert!(typing.notices.stopped.is_empty());
    // The widget reports that it lost the keyboard after the click that left it
    let left = after(typing, [Message::Clicked, Message::Held(Held::default())]);
    assert!(left.workspace.active_of(CUE).is_none());
    assert_eq!(left.workspace.cue(CUE).unwrap().file, "bell.ogg");
    assert_eq!(left.notices.stopped, [CUE]);
}

#[test]
fn the_file_of_a_cue_that_stopped_tells_that_it_did_only_for_the_change_that_stopped_it() {
    let left = after(
        typed_file(playing(), "bell.ogg"),
        [Message::Held(Held::default()), Message::Clicked],
    );
    let again = after(
        typed_file(left, "chime.ogg"),
        [Message::Held(Held::default()), Message::Clicked],
    );
    assert_eq!(again.workspace.cue(CUE).unwrap().file, "chime.ogg");
    assert!(again.notices.stopped.is_empty());
}

#[test]
fn leaving_the_file_of_a_cue_unchanged_makes_no_change() {
    let typing = typed_file(playing(), "bell.ogg");
    let back = typed_setting(typing, Setting::File(CUE), "sound.ogg");
    let left = after(back, [Message::Held(Held::default()), Message::Clicked]);
    assert!(left.workspace.active_of(CUE).is_some());
    assert!(left.notices.stopped.is_empty());
}
