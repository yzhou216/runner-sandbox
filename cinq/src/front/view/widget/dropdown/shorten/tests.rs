// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the names cut short to fit their room

use super::{shortened, shortened_start};

#[test]
fn name_too_long_ends_in_an_ellipsis_within_its_columns() {
    let cases = [
        ("Booth monitors", 14, "Booth monitors"),
        ("Booth monitors", 13, "Booth monito\u{2026}"),
        ("HDA Intel PCH, ALC256 Analog", 8, "HDA Int\u{2026}"),
        // Wide characters take two columns each
        ("\u{97f3}\u{97ff}\u{5353}", 4, "\u{97f3}\u{2026}"),
        // Wide name that would end past an odd room stops a column short
        (
            "\u{65e5}\u{672c}\u{8a9e}\u{306e}\u{53f0}\u{672c}.cinq.toml",
            9,
            "\u{65e5}\u{672c}\u{8a9e}\u{306e}\u{2026}",
        ),
        ("show.cinq.toml", 1, "\u{2026}"),
        ("Default device", 0, "\u{2026}"),
    ];
    cases.into_iter().for_each(|(name, columns, expected)| {
        assert_eq!(shortened(name, columns), expected, "{name} in {columns}");
    });
}

#[test]
fn name_cut_at_its_start_keeps_the_end_within_its_columns() {
    let cases = [
        ("Booth monitors", 14, "Booth monitors"),
        ("Booth monitors", 13, "\u{2026}oth monitors"),
        // Wide characters take two columns each
        ("\u{97f3}\u{97ff}\u{5353}", 4, "\u{2026}\u{5353}"),
        ("show.cinq.toml", 1, "\u{2026}"),
        ("Default device", 0, "\u{2026}"),
    ];
    cases.into_iter().for_each(|(name, columns, expected)| {
        assert_eq!(
            shortened_start(name, columns),
            expected,
            "{name} in {columns}"
        );
    });
}

#[test]
fn name_cut_at_its_start_never_begins_with_a_mark_that_lost_its_letter() {
    // Accent after its letter takes no column, so the cut that drops the
    // letter must drop the accent too
    assert_eq!(shortened_start("abe\u{301}f", 2), "\u{2026}f");
    assert_eq!(shortened_start("abe\u{301}f", 3), "\u{2026}e\u{301}f");
}

#[test]
fn name_with_control_characters_never_outgrows_its_columns() {
    use unicode_width::UnicodeWidthStr;
    // A tab takes a column in the width of a string, as the check that a
    // name needs no cut counts it
    let tabs = "\t".repeat(10);
    let cut = shortened(&format!("{tabs}abcdef"), 5);
    assert!(cut.width() <= 5, "{cut:?}");
    let cut = shortened_start(&format!("abcdef{tabs}"), 5);
    assert!(cut.width() <= 5, "{cut:?}");
}
