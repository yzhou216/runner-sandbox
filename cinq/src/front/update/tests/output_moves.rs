// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the notice of a move of an output, which names the output that
//! moved and not a neighbor that it passed

use super::{after, output_editing::on, told};
use crate::front::Message;

#[test]
fn a_move_names_the_output_that_moved_whichever_way_and_however_far() {
    // Front fill is the fifth output, between Monitor and Balcony
    for by in [1, 2, -1, -2] {
        let moved = after(on(5), [Message::Move(by)]);
        assert_eq!(told(&moved), "moved the output Front fill", "by {by}");
        let undone = after(moved, [Message::Undo(1)]);
        assert_eq!(
            told(&undone),
            "moved the output Front fill",
            "undone, by {by}"
        );
    }
    // Neighbor that is selected afterwards is not the one that moved
    let moved = after(on(5), [Message::Move(-2), Message::Row(3)]);
    assert_eq!(told(&moved), "moved the output Front fill");
    // Moved to the first place, it names the output beside what that did
    let first = after(on(3), [Message::Move(-2)]);
    assert_eq!(told(&first), "moved Sub");
}
