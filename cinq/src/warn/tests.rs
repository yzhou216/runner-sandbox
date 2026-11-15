// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of when a warning comes out

use super::Warnings;

#[test]
fn warning_comes_out_once_after_cinq() {
    let (warnings, first) = Warnings::default().warned("a.mp4: skipped its tags");
    assert_eq!(first, "cinq: a.mp4: skipped its tags\n");
    let (warnings, again) = warnings.warned("a.mp4: skipped its tags");
    assert_eq!(again, "");
    let (_, other) = warnings.warned("b.mp4: skipped its tags");
    assert_eq!(other, "cinq: b.mp4: skipped its tags\n");
}
