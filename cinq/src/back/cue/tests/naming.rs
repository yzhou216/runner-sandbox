// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how a cue reads as text and takes its name from a file

use super::*;

#[test]
fn cue_without_a_number_displays_as_its_name_alone() {
    let numbered = audio("Rain", "rain.wav");
    let unnumbered = Cue::new(CueId(2), " ", Kind::Audio, "Rain").build();
    assert_eq!(numbered.to_string(), "1 Rain");
    assert_eq!(unnumbered.to_string(), "Rain");
}

#[test]
fn chosen_sound_file_names_a_cue_until_the_operator_names_it() {
    let chosen = |cue: Cue| cue.playing("sounds/thunder.wav".to_owned());
    let thunder = Edit::FileAndName {
        file: "sounds/thunder.wav".to_owned(),
        name: "thunder".to_owned(),
    };
    assert_eq!(chosen(audio(&Cue::new_name(Kind::Audio), "")), thunder);
    assert_eq!(chosen(audio("storm", "sounds/storm.ogg")), thunder);
    assert_eq!(
        chosen(audio("Storm build", "sounds/storm.ogg")),
        Edit::File("sounds/thunder.wav".to_owned())
    );
}

#[test]
fn a_name_with_line_breaks_reads_on_one_line() {
    let cue = Cue::new(CueId(1), "4", Kind::Audio, "Two\nlines\t name").build();
    assert_eq!(cue.to_string(), "4 Two lines name");
    assert_eq!(one_line("a\n\n  \nb\r\n"), "a b");
}
