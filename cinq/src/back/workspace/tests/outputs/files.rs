// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the outputs and sends in show files, and of the problems that a
//! file with a bad output shows

use super::*;
use crate::back::Device;

#[test]
fn show_file_holds_outputs_and_sends_by_name_and_main_alone_as_before() {
    assert!(!workspace(vec![audio(1)]).show(None).contains("[[output]]"));
    let named = Device::Named {
        id: "alsa:plughw:CARD=X32,DEV=0".to_owned(),
        name: "X32, USB Audio".to_owned(),
    };
    let show = house_and_sub()
        .apply(edit_output(2, OutputEdit::Device(named)))
        .apply(send_on(&[(2, -3.0)]));
    let text = show.show(None);
    assert!(text.contains("name = \"Sub\""), "{text}");
    assert!(text.contains("channels = \"3\""), "{text}");
    assert!(text.contains("device_name = \"X32, USB Audio\""), "{text}");
    assert!(text.contains("Sub = -3.0"), "{text}");
    let read = Workspace::from_show(&text, "/").expect("the show reads back");
    assert_eq!(read.outputs(), show.outputs());
    assert_eq!(sends_of_cue(&read), sends(&[(2, -3.0)]));
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
        (output("  ", "1-2"), "an output has no name"),
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

#[test]
fn an_output_that_lacks_a_field_is_named_by_its_place_and_its_line() {
    let output = |channels: &str| format!("[[output]]\nname = \"Out\"\n{channels}\n");
    let text = output("channels = \"1-2\"")
        + "\n"
        + &output("channels = \"3\"")
        + "\n[[output]]\nname = \"Sub\"\n";
    assert_eq!(
        Workspace::from_show(&text, "/").err().as_deref(),
        Some("line 9, column 1: output 3: missing field `channels`")
    );
    let first = "[[output]]\nname = \"Main\"\n";
    assert_eq!(
        Workspace::from_show(first, "/").err().as_deref(),
        Some("line 1, column 1: output 1: missing field `channels`")
    );
}

#[test]
fn an_output_after_one_whose_head_has_a_comment_is_named_by_its_place() {
    let text =
        "[[output]] # main\nname = \"Main\"\nchannels = \"1-2\"\n\n[[output]]\nname = \"Sub\"\n";
    assert_eq!(
        Workspace::from_show(text, "/").err().as_deref(),
        Some("line 5, column 1: output 2: missing field `channels`")
    );
}
