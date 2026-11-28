// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that outlast the words of a notice that is cut

use super::*;

#[test]
fn tab_outlasts_the_words_of_a_notice_where_no_key_cancels_in_a_checkbox_or_list() {
    let outlasting = |keymap, text_held| {
        order::outlasting(&held(keymap, text_held)).contains(&Action::NextField)
    };
    assert!(outlasting(desk(""), false));
    assert!(!outlasting(desk(""), true), "Enter leaves a text field");
    assert!(
        !outlasting(Keymap::default(), false),
        "the key of cancel leaves"
    );
}

#[test]
fn tab_outlasts_the_words_of_a_notice_in_the_empty_file_field_where_enter_chooses() {
    let outlasting = |file: &str, text_held| {
        let keyboard = Keyboard::default().with_held(Held {
            any: true,
            file: true,
            text: text_held,
            ..Held::default()
        });
        order::outlasting(&holding_file(desk(""), file, keyboard)).contains(&Action::NextField)
    };
    assert!(outlasting("", true));
    assert!(!outlasting("rain.flac", true), "Enter leaves a named file");
}

#[test]
fn the_key_that_leaves_a_field_outlasts_the_words_of_a_notice_that_is_cut() {
    let told = "Changed the file of cue 13";
    let hints = before([GO, (Action::Cancel, "Control+c: leave")]);
    let shown = |outlasting: &[Action], room| {
        shown(
            &hints,
            Some(told.to_owned()),
            room,
            &YIELDING_IN_FIELD,
            outlasting,
        )
    };
    // The notice cut to the cue keeps the key, where the key would go first otherwise
    let room = "Changed… cue 13 | Control+c: leave".width();
    assert_eq!(
        shown(&[Action::Cancel], room),
        "Changed… cue 13 | Control+c: leave"
    );
    assert!(!shown(&[], room).contains("leave"));
}

#[test]
fn a_notice_that_would_keep_under_three_words_gives_its_room_to_the_keys() {
    let told = "Added cue 12";
    let hints = before([(Action::Cancel, "Control+c: leave")]);
    let shown = |room| {
        shown(
            &hints,
            Some(told.to_owned()),
            room,
            &YIELDING_IN_FIELD,
            &[Action::Cancel],
        )
    };
    assert_eq!(shown(31), "Added cue 12 | Control+c: leave");
    assert_eq!(
        shown(20),
        "Control+c: leave",
        "no stub such as `Added cue…`"
    );
}

#[test]
fn a_notice_that_is_cut_keeps_the_words_that_tell_which_cue_before_those_that_do_not() {
    let hints = before([(Action::Redo, "Control+r: redo")]);
    let shown =
        |told: &str, room| shown(&hints, Some(told.to_owned()), room, &YIELDING_IN_FIELD, &[]);
    let told = "Undid: changed the number of cue 6.5";
    let key = " | Control+r: redo";
    let room = |text: &str| text.width() + key.width();
    assert_eq!(shown(told, room(told)), format!("{told}{key}"));
    // Dropping the words before the last two, as many as the room asks
    assert_eq!(
        shown(told, room("Undid: changed the number… cue 6.5")),
        "Undid: changed the number… cue 6.5 | Control+r: redo"
    );
    assert_eq!(
        shown(told, room("Undid: changed… cue 6.5") + 1),
        "Undid: changed… cue 6.5 | Control+r: redo"
    );
    // Room for a verb alone before the ellipsis still keeps the cue
    assert_eq!(
        shown(told, room("Undid: changed… cue 6.5") - 1),
        "Undid… cue 6.5 | Control+r: redo"
    );
    // Notices that name no cue are cut as before, and so are those of a cue
    // whose words fit whole
    assert_eq!(
        shown("Moved standby past the end", room("Moved standby past…")),
        "Moved standby past… | Control+r: redo"
    );
    assert_eq!(
        shown("Moved cue 13 into a group", room("Moved cue 13 into a…")),
        "Moved cue 13 into a… | Control+r: redo"
    );
}

#[test]
fn what_enter_does_outlasts_the_words_of_a_notice_that_is_cut() {
    for (keymap, room) in [(Keymap::default(), 52), (desk(""), 62)] {
        let app = App {
            workspace: added(),
            ..held(keymap, false)
        };
        let shown = hints_in(&app, room);
        assert!(shown.contains("Enter: open"), "{shown}");
    }
}
