// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the notices that cuts, pastes, adds, and changes of the first
//! output show

use super::*;

#[test]
fn a_cut_of_the_first_output_tells_where_cues_with_no_sends_play_now() {
    let cut = typed(on(1), "dd");
    assert_eq!(
        on_clipboard(&cut),
        Some(("Cues without sends play on House R, cut House L", true))
    );
    // Not when the first output stays or no cue is left without sends
    let kept = typed(on(2), "dd");
    assert_eq!(on_clipboard(&kept), Some(("Cut the output House R", true)));
}

#[test]
fn a_change_of_the_first_output_tells_it_in_the_notice_of_the_change() {
    let moved = after(on(1), [Message::Move(1)]);
    assert_eq!(
        noticed(&moved),
        "Cues without sends play on House R, moved House L"
    );
    let added = after(on(1), [Message::Add(Place::Before, 1)]);
    assert_eq!(
        noticed(&added),
        "Cues without sends play on Output 1, added Output 1"
    );
    let deleted = after(on(1), [Message::Delete(1)]);
    assert_eq!(
        noticed(&deleted),
        "Cues without sends play on House R, deleted House L"
    );
    let calm = after(on(3), [Message::Move(1)]);
    assert_eq!(noticed(&calm), "Moved the output Sub");
    let counted = after(on(3), [Message::Add(Place::After, 3)]);
    assert_eq!(noticed(&counted), "Added 3 outputs");
}

#[test]
fn a_cut_by_keys_is_told_as_a_cut_by_the_change_and_a_delete_as_a_delete() {
    assert_eq!(told(&typed(on(3), "dd")), "cut the output Sub");
    assert_eq!(told(&typed(on(3), "2dj")), "cut 3 outputs");
    assert_eq!(
        told(&after(on(3), [Message::Delete(1)])),
        "deleted the output Sub"
    );
    // Leaving the outputs does not turn the cut into a delete
    let left = after(typed(on(3), "dd"), [Message::Back(1)]);
    assert_eq!(told(&left), "cut the output Sub");
}

#[test]
fn a_paste_by_keys_is_told_as_a_paste_and_an_add_as_an_add() {
    assert_eq!(told(&typed(on(3), "ddp")), "pasted the output Sub");
    assert_eq!(told(&typed(on(3), "yy2p")), "pasted 2 outputs");
    assert_eq!(
        told(&after(on(3), [Message::Add(Place::After, 1)])),
        "added the output Output 1"
    );
}
