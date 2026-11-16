// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the State column of the cue list

use iced::widget::text;

use super::state;
use crate::{
    back::{Command, Cue, CueId, CueList, Edit, Kind, Media, OutputId, Sends, Workspace},
    front::view::theme,
};

fn show(sends: Option<Sends>) -> Workspace {
    let cue = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .file("rain.ogg")
        .build();
    Workspace::new(vec![CueList::new("Main", vec![cue])])
        .apply(Command::Loaded {
            cue: CueId(1),
            file: "rain.ogg".to_owned(),
            media: Media::Loaded {
                duration: 10.0,
                sound: Some(10.0),
            },
        })
        .apply(Command::AddOutput {
            at: 1,
            count: 1,
            anchor: None,
        })
        .apply(Command::Edit(CueId(1), Edit::Sends(sends)))
        .apply(Command::DeleteOutputs(vec![OutputId(2)]))
}

fn shown(workspace: &Workspace) -> (String, bool) {
    let list = &workspace.lists()[0];
    let cue = &list.cues[0];
    let (state, style) = state(workspace, list, cue, false);
    let theme = theme::modus_vivendi();
    let yellow = style(&theme).color == text::warning(&theme).color;
    (state, yellow)
}

#[test]
fn a_cue_that_sends_to_no_output_left_reads_in_yellow_that_it_does() {
    let sends = [(OutputId(2), 0.0)].into();
    assert_eq!(shown(&show(Some(sends))), ("NO OUTPUT".to_owned(), true));
}

#[test]
fn a_cue_that_sends_somewhere_or_names_no_sends_reads_as_any_other() {
    let both: Sends = [(OutputId(1), 0.0), (OutputId(2), 0.0)].into();
    assert_eq!(shown(&show(Some(both))).0, "NEXT");
    assert_eq!(shown(&show(None)).0, "NEXT");
}
