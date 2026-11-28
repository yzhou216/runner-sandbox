// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how a typed level or send level reads

use super::*;

#[test]
fn typed_level_reads_with_or_without_its_unit() {
    assert_eq!(parse_level("-6"), Some(-6.0));
    assert_eq!(parse_level(" +3.5 dB "), Some(3.5));
    assert_eq!(parse_level("-60db"), Some(-60.0));
}

#[test]
fn typed_level_past_either_end_or_half_typed_reads_as_none() {
    assert_eq!(parse_level("-"), None);
    assert_eq!(parse_level("-61"), None);
    assert_eq!(parse_level("13"), None);
    assert_eq!(parse_level("NaN"), None);
}

#[test]
fn silence_is_where_kira_stops_making_sound() {
    assert_eq!(SILENCE, kira::Decibels::SILENCE.0);
}

#[test]
fn minus_infinity_reads_as_silence() {
    assert_eq!(parse_level("-inf"), Some(SILENCE));
    assert_eq!(parse_level(" -INF dB "), Some(SILENCE));
    assert_eq!(parse_level("inf"), None);
}

#[test]
fn send_level_reads_off_as_no_send() {
    assert_eq!(parse_send("off"), Some(None));
    assert_eq!(parse_send(" Off "), Some(None));
    assert_eq!(parse_send(""), Some(None));
    assert_eq!(parse_send("-6"), Some(Some(-6.0)));
    assert_eq!(parse_send("-inf"), Some(Some(SILENCE)));
    assert_eq!(parse_send("-61"), None);
}
