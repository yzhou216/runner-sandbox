// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of a paste into another show: the sends and lights that it cannot
//! take

use super::*;

/// Returns a show of one empty list with outputs of the `names`, which take
/// identities from one in that order
fn with_outputs(cues: Vec<Cue>, names: &[&str]) -> Workspace {
    names.iter().enumerate().fold(
        fresh(cues).apply(Command::add_output(1, names.len() - 1)),
        |workspace, (row, name)| {
            let id = workspace.outputs()[row].id;
            workspace.apply(Command::EditOutput(
                id,
                OutputEdit::Name((*name).to_owned()),
            ))
        },
    )
}

/// Returns the state after a cue sending to House and Sub is copied and
/// pasted into another show, one with outputs of the `names`
fn pasted_app(names: &[&str]) -> App {
    let sends: Sends = [(OutputId(1), -3.0), (OutputId(2), -6.0)].into();
    let cue = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .sends(sends)
        .build();
    let copied = after(
        App {
            workspace: with_outputs(vec![cue], &["House", "Sub"]),
            selected: Some(CueId(1)),
            ..App::default()
        },
        [Message::Copy(1)],
    );
    let elsewhere = App {
        workspace: with_outputs(Vec::new(), names),
        clipboard: Clipboard {
            show: copied.show.id().wrapping_add(1),
            ..copied.clipboard
        },
        selected: None,
        ..copied
    };
    after(elsewhere, [Message::Paste(Place::After, 1)])
}

#[test]
fn a_paste_sends_to_the_outputs_of_the_same_names_and_tells_what_it_dropped() {
    // Names of the outputs of the other show, the sends that the pasted cue
    // keeps, and what the status bar tells.  Sub is there in the other show,
    // first, while House is not; where no output has a name, the cue plays on
    // the first output.
    let rows: [(&[&str], Option<Sends>, Option<&str>); 3] = [
        (
            &["Sub", "Foyer"],
            Some([(OutputId(1), -6.0)].into()),
            Some("Pasted 1 cue, dropped 1 send"),
        ),
        (&["Foyer"], None, Some("Pasted 1 cue, dropped 2 sends")),
        (
            &["House", "Sub"],
            Some([(OutputId(1), -3.0), (OutputId(2), -6.0)].into()),
            None,
        ),
    ];
    rows.into_iter().for_each(|(names, sends, told)| {
        let app = pasted_app(names);
        assert_eq!(app.workspace.lists()[0].cues[0].sends, sends, "{names:?}");
        assert_eq!(
            on_clipboard(&app),
            told.map(|told| (told, true)),
            "{names:?}"
        );
    });
}

#[test]
fn a_paste_tells_the_light_cues_that_the_other_patch_lacks() {
    let cue = Cue {
        lights: [(InstrumentId(7), 50.0)].into(),
        ..Cue::new(CueId(1), "1", Kind::Light, "").build()
    };
    let copied = after(one_list(vec![cue], Some(CueId(1))), [Message::Copy(1)]);
    let elsewhere = App {
        workspace: Workspace::default(),
        clipboard: Clipboard {
            show: copied.show.id().wrapping_add(1),
            ..copied.clipboard
        },
        selected: None,
        ..copied
    };
    let pasted = after(elsewhere, [Message::Paste(Place::After, 1)]);
    assert_eq!(
        on_clipboard(&pasted),
        Some(("Pasted 1 cue, 1 unpatched", true))
    );
}
