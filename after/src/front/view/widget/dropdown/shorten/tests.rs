// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the names that a list cuts short to fit its room

use super::shortened;

#[test]
fn name_too_long_ends_in_an_ellipsis_within_its_columns() {
    let cases = [
        ("Booth monitors", 14, "Booth monitors"),
        ("Booth monitors", 13, "Booth monito\u{2026}"),
        ("HDA Intel PCH, ALC256 Analog", 8, "HDA Int\u{2026}"),
        // Wide characters take two columns each
        ("\u{97f3}\u{97ff}\u{5353}", 4, "\u{97f3}\u{2026}"),
        ("Default device", 0, "\u{2026}"),
    ];
    let wrong: Vec<_> = cases
        .into_iter()
        .map(|(name, columns, expected)| (name, columns, expected, shortened(name, columns)))
        .filter(|(.., expected, got)| expected != got)
        .collect();
    assert!(wrong.is_empty(), "{wrong:?}");
}
