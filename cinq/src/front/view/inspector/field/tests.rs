// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which fields the inspector shows for a cue, which it tells the
//! cue ignores, and which time it says reads as no time

use super::{Field, unread};
use crate::back::{Advance, Cue, CueId, Kind};

fn cue(kind: Kind) -> Cue {
    Cue::new(CueId(1), "1", kind, "Cue").build()
}

/// Returns an audio cue that goes on to the next cue by itself
fn following() -> Cue {
    Cue {
        advance: Advance::Follow,
        ..cue(Kind::Audio)
    }
}

#[test]
fn a_kind_with_no_file_and_no_target_has_no_target_row() {
    [Kind::Light, Kind::Group, Kind::Wait, Kind::Memo]
        .into_iter()
        .for_each(|kind| assert!(!Field::Target.has_row(&cue(kind))));
    [Kind::Audio, Kind::Network, Kind::Stop]
        .into_iter()
        .for_each(|kind| assert!(Field::Target.has_row(&cue(kind))));
    assert!(Field::Notes.has_row(&cue(Kind::Memo)));
}

#[test]
fn a_network_cue_puts_its_message_in_the_wide_form_and_its_notes_in_the_narrow_one() {
    let network = cue(Kind::Network);
    assert!(matches!(Field::Notes.of(&network), Field::Osc));
    assert!(matches!(Field::Duration.of(&network), Field::Notes));
    assert!(matches!(Field::Notes.of(&cue(Kind::Audio)), Field::Notes));
    assert!(matches!(
        Field::Duration.of(&cue(Kind::Audio)),
        Field::Duration
    ));
}

#[test]
fn a_cue_tells_why_it_ignores_a_field() {
    let following = following();
    assert_eq!(
        Field::PostWait.ignored(&following, false),
        Some("with auto-follow")
    );
    assert_eq!(
        Field::PostWait.ignored(&cue(Kind::Audio), true),
        Some("in a group")
    );
    assert_eq!(
        Field::Then.ignored(&cue(Kind::Audio), true),
        Some("in a group")
    );
    assert_eq!(
        Field::LoopStart.ignored(&cue(Kind::Audio), false),
        Some("not looping")
    );
    assert_eq!(Field::PreWait.ignored(&following, true), None);
}

#[test]
fn the_first_time_that_reads_as_no_time_is_named() {
    let columns: [&[Field]; 2] = [&[Field::Name], &[Field::PreWait, Field::PostWait]];
    assert_eq!(unread(&cue(Kind::Audio), false, &columns), None);
    let typed = Cue {
        pre_wait: "1,5".to_owned().into(),
        post_wait: "abc".to_owned().into(),
        ..cue(Kind::Audio)
    };
    assert_eq!(
        unread(&typed, false, &columns).as_deref(),
        Some("Pre-wait counts as zero: use seconds or minutes:seconds")
    );
}

#[test]
fn a_time_that_the_cue_ignores_is_not_named_where_it_reads_as_no_time() {
    let columns: [&[Field]; 1] = [&[Field::PostWait, Field::LoopStart, Field::LoopEnd]];
    let typed = Cue {
        post_wait: "abc".to_owned().into(),
        loop_start: "abc".to_owned().into(),
        loop_end: "abc".to_owned().into(),
        advance: Advance::Follow,
        looping: false,
        ..cue(Kind::Audio)
    };
    assert_eq!(unread(&typed, false, &columns), None);
    let in_group = Cue {
        advance: Advance::Continue,
        looping: true,
        ..typed
    };
    assert_eq!(
        unread(&in_group, true, &columns).as_deref(),
        Some("Loop start reads as empty: use seconds or minutes:seconds")
    );
    assert_eq!(
        unread(&in_group, false, &columns).as_deref(),
        Some("Post-wait counts as zero: use seconds or minutes:seconds")
    );
}

#[test]
fn an_end_or_a_loop_time_that_reads_as_no_time_is_told_as_empty() {
    let columns: [&[Field]; 1] = [&[Field::End, Field::LoopStart, Field::LoopEnd]];
    let words = |typed: Cue| unread(&typed, false, &columns);
    let empty = "reads as empty: use seconds or minutes:seconds";
    let end = Cue {
        end: "5s".to_owned().into(),
        ..cue(Kind::Audio)
    };
    assert_eq!(words(end), Some(format!("End {empty}")));
    let loop_start = Cue {
        loop_start: "5s".to_owned().into(),
        looping: true,
        ..cue(Kind::Audio)
    };
    assert_eq!(words(loop_start), Some(format!("Loop start {empty}")));
    let loop_end = Cue {
        loop_end: "5s".to_owned().into(),
        looping: true,
        ..cue(Kind::Audio)
    };
    assert_eq!(words(loop_end), Some(format!("Loop end {empty}")));
}

#[test]
fn minutes_and_seconds_an_empty_field_and_a_duration_the_cue_does_not_hold_are_no_time_to_tell() {
    let columns: [&[Field]; 2] = [&[Field::PreWait], &[Field::PostWait]];
    let typed = Cue {
        pre_wait: "1:30".to_owned().into(),
        post_wait: String::new().into(),
        ..cue(Kind::Audio)
    };
    assert_eq!(unread(&typed, false, &columns), None);
    let columns: [&[Field]; 2] = [&[Field::Duration], &[]];
    let memo = Cue {
        duration: "abc".to_owned().into(),
        ..cue(Kind::Memo)
    };
    assert_eq!(unread(&memo, false, &columns), None);
}

#[test]
fn a_reason_fits_the_box_beside_the_longest_value_at_1024_pixels() {
    // The box of the Then setting at 1024x768 shows about 30 characters
    // of the longest value, "wait for GO", with its reason in parentheses
    [
        Field::PostWait.ignored(&following(), false),
        Field::Then.ignored(&cue(Kind::Audio), true),
        Field::LoopEnd.ignored(&cue(Kind::Audio), false),
    ]
    .into_iter()
    .for_each(|reason| {
        let reason = reason.expect("a reason");
        assert!("wait for GO ".len() + reason.len() + 2 <= 30, "{reason}");
    });
}

#[test]
fn the_empty_destination_of_a_network_cue_names_both_forms() {
    let placeholder = Field::Target.placeholder(&cue(Kind::Network));
    assert!(placeholder.contains("host:port"));
    assert!(placeholder.contains("udp://host:port"));
}

#[test]
fn the_target_menu_does_not_invite_a_choice_where_there_is_none() {
    assert_eq!(super::target::target_placeholder(false), "choose a cue");
    assert_eq!(super::target::target_placeholder(true), "no cue to choose");
}

#[test]
fn the_then_box_of_a_cue_in_a_group_gives_only_the_reason() {
    assert_eq!(super::then_text(&following(), true), "");
    assert_eq!(
        super::then_text(&following(), false),
        Advance::Follow.to_string()
    );
}
