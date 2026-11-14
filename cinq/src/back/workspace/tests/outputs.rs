// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the outputs of a show, of the sends of its cues to them, and of
//! both in show files

use super::*;
use crate::back::{Channels, Device, OutputEdit, OutputId, Sends};

const MAIN: OutputId = OutputId(1);

fn names(workspace: &Workspace) -> Vec<&str> {
    workspace
        .outputs()
        .iter()
        .map(|output| output.name.as_str())
        .collect()
}

fn edit_output(id: u32, edit: OutputEdit) -> Command {
    Command::EditOutput(OutputId(id), edit)
}

fn sends(pairs: &[(u32, f32)]) -> Sends {
    pairs
        .iter()
        .map(|&(id, level)| (OutputId(id), level))
        .collect()
}

/// Returns a show with House on the first two channels and Sub on the
/// third, to which new cues send at 0 and -6 dB
fn house_and_sub() -> Workspace {
    workspace(vec![audio(1)])
        .apply(edit_output(1, OutputEdit::Name("House".to_owned())))
        .apply(Command::AddOutput { at: 1 })
        .apply(edit_output(2, OutputEdit::Name("Sub".to_owned())))
        .apply(edit_output(2, OutputEdit::Channels(Channels::Mono(2))))
        .apply(edit_output(2, OutputEdit::NewCues(Some(-6.0))))
}

#[test]
fn show_starts_with_main_where_cues_that_name_no_outputs_send() {
    let show = workspace(vec![audio(1)]);
    assert_eq!(names(&show), ["Main"]);
    let cue = show.cue(CueId(1)).expect("the cue is there");
    assert_eq!(show.sends(cue), sends(&[(1, 0.0)]));
}

#[test]
fn outputs_add_after_one_another_under_names_of_their_own_and_keep_one() {
    let added = workspace(Vec::new())
        .apply(Command::AddOutput { at: usize::MAX })
        .apply(Command::AddOutput { at: 1 });
    assert_eq!(names(&added), ["Main", "Output 2", "Output 1"]);
    let renamed = added
        .apply(edit_output(2, OutputEdit::Name("Main".to_owned())))
        .apply(edit_output(3, OutputEdit::Name(" ".to_owned())));
    assert_eq!(names(&renamed), ["Main", "Output 2", "Output 1"]);
    let moved = renamed.apply(Command::MoveOutput {
        output: MAIN,
        by: 5,
    });
    assert_eq!(names(&moved), ["Output 2", "Output 1", "Main"]);
    let one = [MAIN, OutputId(2), OutputId(3)]
        .into_iter()
        .fold(moved, |show, id| show.apply(Command::DeleteOutput(id)));
    assert_eq!(names(&one), ["Output 2"]);
}

#[test]
fn new_cues_send_where_the_outputs_say_and_a_name_typed_out_undoes_at_once() {
    let show = house_and_sub().apply(add(0, Kind::Audio, Place::End));
    let new = show.cues().last().expect("the cue is added");
    assert_eq!(show.sends(new), sends(&[(1, 0.0), (2, -6.0)]));
    let typed = "Lobby"
        .char_indices()
        .map(|(end, _)| &"Lobby"[..=end])
        .fold(show, |show, name| {
            show.apply(edit_output(2, OutputEdit::Name(name.to_owned())))
        });
    assert_eq!(names(&typed), ["House", "Lobby"]);
    let undone = typed.apply(Command::Undo);
    assert_eq!(names(&undone), ["House", "Sub"]);
}

#[test]
fn deleting_an_output_silences_its_sends_until_undo_brings_it_back() {
    let show = house_and_sub().apply(edit(1, Edit::Sends(Some(sends(&[(1, 0.0), (2, -3.0)])))));
    let deleted = show.apply(Command::DeleteOutput(OutputId(2)));
    let cue = |show: &Workspace| show.cue(CueId(1)).cloned().expect("the cue is there");
    assert_eq!(deleted.sends(&cue(&deleted)), sends(&[(1, 0.0)]));
    let undone = deleted.apply(Command::Undo);
    assert_eq!(undone.sends(&cue(&undone)), sends(&[(1, 0.0), (2, -3.0)]));
}

#[test]
fn show_file_holds_outputs_and_sends_by_name_and_main_alone_as_before() {
    assert!(!workspace(vec![audio(1)]).show(None).contains("[[output]]"));
    let named = Device::Named {
        id: "alsa:plughw:CARD=X32,DEV=0".to_owned(),
        name: "X32, USB Audio".to_owned(),
    };
    let show = house_and_sub()
        .apply(edit_output(2, OutputEdit::Device(named)))
        .apply(edit(1, Edit::Sends(Some(sends(&[(2, -3.0)])))));
    let text = show.show(None);
    assert!(text.contains("name = \"Sub\""), "{text}");
    assert!(text.contains("channels = \"3\""), "{text}");
    assert!(text.contains("device_name = \"X32, USB Audio\""), "{text}");
    assert!(text.contains("Sub = -3.0"), "{text}");
    let read = Workspace::from_show(&text, "/").expect("the show reads back");
    assert_eq!(read.outputs(), show.outputs());
    let cue = read.cue(CueId(1)).expect("the cue reads back");
    assert_eq!(read.sends(cue), sends(&[(2, -3.0)]));
}

#[test]
fn show_file_with_outputs_that_cannot_play_cannot_be_read() {
    let output = |name: &str, channels: &str| {
        format!("[[output]]\nname = \"{name}\"\nchannels = \"{channels}\"\n")
    };
    let cue = "[[list]]\nname = \"Main\"\n[[list.cue]]\nid = 1\nkind = \"audio\"\n";
    let cases = [
        (
            output("House", "1-2") + &output("House", "3"),
            "more than one output has the name House",
        ),
        (
            output("House", "1-3"),
            "the output House plays on the channels 1-3, which are neither one channel nor \
             two, counted from 1",
        ),
        (
            output("House", "1-2") + cue + "[list.cue.sends]\nLobby = 0.0\n",
            "the cue with the id 1 sends to Lobby, which no output is named",
        ),
        (
            output("House", "1-2") + cue + "[list.cue.sends]\nHouse = 60.0\n",
            "the cue with the id 1 sends to House at 60 dB, which is not between -60 and 12 dB",
        ),
    ];
    cases.into_iter().for_each(|(text, problem)| {
        assert_eq!(
            Workspace::from_show(&text, "/").err().as_deref(),
            Some(problem),
            "{text}"
        );
    });
}
