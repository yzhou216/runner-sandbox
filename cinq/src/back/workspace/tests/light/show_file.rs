// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the show file with a patch and light cues

use super::*;
use crate::back::PatchEdit;

#[test]
fn show_file_holds_the_patch_and_the_levels_of_light_cues() {
    let show = shown(
        &lit(vec![light(1, &[(2, 50.0)])]).apply(Command::Patch(PatchEdit::Address(
            InstrumentId(2),
            2,
            12,
        ))),
    );
    assert_eq!(
        show,
        r#"[[instrument]]
id = 1
name = "1"
address = 1

[[instrument]]
id = 2
name = "2"
universe = 2
address = 12

[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "light"
name = "Light"
duration = 4.0

[list.cue.lights]
2 = 50.0
"#
    );
}

#[test]
fn show_file_with_an_address_or_a_light_level_out_of_range_cannot_be_read() {
    let instrument = |address| format!("[[instrument]]\nid = 1\naddress = {address}\n");
    assert!(Workspace::from_show(&instrument(512), "/show").is_ok());
    assert!(Workspace::from_show(&instrument(0), "/show").is_err());
    let cue = |level| {
        format!(
            "[[list]]\nname = \"Main\"\n[[list.cue]]\nid = 1\nkind = \"light\"\nlights = {{ 1 = {level} }}\n"
        )
    };
    assert!(Workspace::from_show(&cue(100.0), "/show").is_ok());
    assert!(Workspace::from_show(&cue(100.5), "/show").is_err());
}
