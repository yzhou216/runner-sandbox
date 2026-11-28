// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the notices that count the cues which send to no output once
//! outputs are cut, deleted, or brought back

use super::{
    after, noticed, on_clipboard,
    output_editing::{on, typed},
    typed_setting,
};
use crate::{
    back::{Command, CueId, Edit, OutputId, Sends},
    front::{App, Message, Setting},
};

/// Returns the app whose first cue sends to House L and House R alone
fn sending_to_the_house() -> App {
    let app = on(1);
    let sends: Sends = [(OutputId(1), 0.0), (OutputId(2), -3.0)].into();
    App {
        workspace: app
            .workspace
            .apply(Command::Edit(CueId(1), Edit::Sends(Some(sends)))),
        ..app
    }
}

#[test]
fn a_cut_that_takes_every_output_of_a_cue_counts_the_cues_that_send_nowhere() {
    let cut = typed(sending_to_the_house(), "2dd");
    assert_eq!(
        on_clipboard(&cut),
        Some((
            "1 cue now sends to no output, cues without sends play on Sub, cut 2 outputs",
            true
        ))
    );
    assert_eq!(cut.workspace.stranded().count(), 1);
    // One output of the two is enough for the cue to play
    let part = typed(sending_to_the_house(), "dd");
    assert_eq!(
        on_clipboard(&part),
        Some(("Cues without sends play on House R, cut House L", true))
    );
}

#[test]
fn a_delete_counts_them_too_and_undo_and_redo_tell_only_what_they_bring_about() {
    let deleted = after(sending_to_the_house(), [Message::Delete(2)]);
    assert_eq!(
        noticed(&deleted),
        "1 cue now sends to no output, cues without sends play on Sub, deleted 2 outputs"
    );
    // Undo brings the sound back, and the cue sends nowhere no more
    let undone = typed(deleted, "u");
    assert_eq!(undone.workspace.stranded().count(), 0);
    assert_eq!(
        noticed(&undone),
        "Cues without sends play on House L, deleted 2 outputs"
    );
    let redone = after(undone, [Message::Redo(1)]);
    assert!(noticed(&redone).contains("1 cue now sends to no output"));
}

#[test]
fn cues_that_were_stranded_already_are_not_told_again() {
    let cut = typed(sending_to_the_house(), "2dd");
    let id = cut.workspace.outputs()[0].id;
    let renamed = typed_setting(cut, Setting::OutputName(id), "Sub 2");
    assert_eq!(noticed(&renamed), "Renamed Sub to Sub 2");
    assert!(!noticed(&renamed).contains("nowhere"));
}
