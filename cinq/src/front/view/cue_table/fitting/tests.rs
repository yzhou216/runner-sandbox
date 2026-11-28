// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the widths that the cue list hands out in a narrow room

use super::{CELL_SPACE, fitting};
use crate::{
    back::{Cue, CueId, Kind, fresh},
    front::{
        fixtures::app_of,
        view::{
            char_width,
            cue_table::{Column, NAME_LEAST, STATE_LEAST, STATE_ROOM, layout::Layout},
        },
    },
};

/// Returns the width of every column that a list of short names shows in a
/// room `room` wide, and the room that is left for the name
fn widths(room: f32) -> (Vec<(Column, f32)>, f32) {
    let cues = ["House open", "Preshow music", "Rain"]
        .into_iter()
        .zip(1..)
        .map(|(name, number)| Cue::new(CueId(number), number.to_string(), Kind::Memo, name).build())
        .collect();
    let app = app_of(fresh(cues));
    let layout = Layout::of(app.current());
    let kept: Vec<_> = fitting(&app, &layout, room)
        .into_iter()
        .map(|(column, width)| (column, width.unwrap_or(0.0)))
        .collect();
    let taken: f32 = kept.iter().map(|(_, width)| width + CELL_SPACE).sum();
    (kept, room - taken)
}

#[test]
fn a_narrow_room_leaves_the_name_its_least_or_all_that_the_state_can_spare_and_the_state_a_word() {
    let floor = STATE_LEAST as f32 * char_width();
    let number = widths(1000.0)
        .0
        .iter()
        .find_map(|&(column, width)| (column == Column::Number).then_some(width))
        .expect("the number column shows");
    // Room that leaves the name nothing when the State column keeps a word
    let bare = number + floor + 3.0 * CELL_SPACE;
    (0..=NAME_LEAST + 4).for_each(|name| {
        let (kept, left) = widths(bare + name as f32 * char_width());
        assert!(
            left >= name.min(NAME_LEAST) as f32 * char_width() - 0.01,
            "{name}: {left}"
        );
        kept.iter().for_each(|&(column, width)| match column {
            Column::State => assert!(width >= floor - 0.01, "{name}"),
            Column::Number | Column::Name => {}
            other => assert!(
                left >= NAME_LEAST as f32 * char_width() - 0.01,
                "{name}: {}",
                other.title()
            ),
        });
    });
}

#[test]
fn the_state_column_narrows_only_where_the_name_would_lack_its_least() {
    let state = |room| {
        widths(room)
            .0
            .into_iter()
            .find_map(|(column, width)| (column == Column::State).then_some(width))
            .expect("the State column shows")
    };
    assert_eq!(state(1000.0), STATE_ROOM as f32 * char_width());
    assert!(state(330.0) < STATE_ROOM as f32 * char_width());
}

#[test]
fn the_state_column_keeps_the_longest_word_of_its_states_in_any_room() {
    let narrow = (100..1200)
        .map(|room| (room, widths(room as f32).0))
        .filter_map(|(room, kept)| {
            kept.into_iter()
                .find(|&(column, width)| {
                    column == Column::State && width < STATE_LEAST as f32 * char_width() - 0.01
                })
                .map(|(_, width)| (room, width))
        })
        .next();
    assert_eq!(narrow, None);
    assert!(
        "NO DESTINATION"
            .split(' ')
            .all(|word| word.len() <= STATE_LEAST)
    );
}

#[test]
fn the_state_column_takes_fourteen_columns_only_while_a_row_shows_no_destination() {
    let state = |kind, destination: &str| {
        let cue = Cue {
            destination: destination.to_owned(),
            ..Cue::new(CueId(1), "1", kind, "Ping").build()
        };
        let app = app_of(fresh(vec![cue]));
        let layout = Layout::of(app.current());
        fitting(&app, &layout, 1000.0)
            .into_iter()
            .find_map(|(column, width)| (column == Column::State).then_some(width))
            .flatten()
            .expect("the State column shows")
    };
    let columns = |width: f32| (width / char_width()).round() as usize;
    assert_eq!(columns(state(Kind::Memo, "")), STATE_ROOM);
    assert_eq!(columns(state(Kind::Network, "")), "NO DESTINATION".len());
    assert_eq!(
        columns(state(Kind::Network, "udp://127.0.0.1:9000")),
        "NO MESSAGE".len().max(STATE_ROOM)
    );
}
