// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which texts a search takes for each other

use super::matching;

fn finds(name: &str, typed: &str) -> bool {
    matching(&[vec![name.to_owned()]], typed) == 1
}

#[test]
fn greek_capitals_match_whatever_part_of_the_word_is_typed() {
    // The capital sigma that ends a text is no final sigma until the word
    // ends, and a search for the part before it must still find the word
    assert!(finds(
        "\u{39f}\u{394}\u{3a5}\u{3a3}\u{3a3}\u{395}\u{3a5}\u{3a3}",
        "\u{39f}\u{394}\u{3a5}\u{3a3}"
    ));
    assert!(finds(
        "\u{3bf}\u{3b4}\u{3c5}\u{3c3}\u{3c3}\u{3b5}\u{3c5}\u{3c2}",
        "\u{39f}\u{394}\u{3a5}\u{3a3}\u{3a3}\u{395}\u{3a5}\u{3a3}"
    ));
}

#[test]
fn dotted_and_dotless_i_and_sharp_s_match_their_plain_letters() {
    assert!(finds("\u{130}stanbul", "istanbul"));
    assert!(finds("ISPARTA", "\u{131}sparta"));
    assert!(finds("Stra\u{df}e", "STRASSE"));
    assert!(finds("STRASSE", "stra\u{df}e"));
}

#[test]
fn letters_that_differ_beyond_their_case_do_not_match() {
    assert!(!finds("Stra\u{df}e", "strase"));
    assert!(!finds("cafe", "caf\u{e9}"));
}
