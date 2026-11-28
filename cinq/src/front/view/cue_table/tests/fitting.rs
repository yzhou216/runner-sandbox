// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the columns that fit and of the room that they leave a name

use strum::VariantArray;

use super::super::{CELL_SPACE, Column, NAME_LEAST, NAME_MOST, fitting, layout::Layout};
use crate::{
    back::{Cue, CueId, CueList, Kind, Workspace, fresh},
    front::{App, view::char_width},
};

/// Returns an app whose one list holds `cue` alone
fn lone(cue: Cue) -> App {
    App {
        workspace: fresh(vec![cue]),
        ..App::default()
    }
}

/// Returns the number of columns that show in `app` and the width in pixels
/// that they take when none gives way
fn whole(app: &App, layout: &Layout) -> (usize, f32) {
    let widths: Vec<f32> = Column::VARIANTS
        .iter()
        .filter(|column| column.shown(&app.workspace, &layout.groups))
        .map(|column| column.width(&app.workspace, &layout.groups).unwrap_or(0.0) + CELL_SPACE)
        .collect();
    (widths.len(), widths.iter().sum())
}

/// Returns the number of columns that a list of one cue of `kind` named `name`
/// shows and the number that it keeps in a room that leaves its name just
/// `room` columns when none gives way
fn columns_named(kind: Kind, name: &str, room: usize) -> (usize, usize) {
    let app = lone(Cue::new(CueId(1), "1", kind, name).build());
    let layout = Layout::of(app.current());
    let (shown, taken) = whole(&app, &layout);
    let room = taken + room as f32 * char_width();
    (shown, fitting(&app, &layout, room).len())
}

/// Returns the columns of a list of one cue of `kind` whose name needs all of
/// [`NAME_MOST`], as [`columns_named`] does
fn columns_kept(kind: Kind) -> (usize, usize) {
    columns_named(kind, &"n".repeat(NAME_MOST), NAME_MOST)
}

#[test]
fn a_list_with_a_group_keeps_the_gutter_of_the_mark_out_of_the_room_of_a_name() {
    let (shown, kept) = columns_kept(Kind::Light);
    assert_eq!(kept, shown);
    let (shown, kept) = columns_kept(Kind::Group);
    assert!(kept < shown);
}

#[test]
fn a_column_that_holds_no_text_in_any_row_is_left_out() {
    let (shown, _) = columns_kept(Kind::Memo);
    assert!(shown < Column::VARIANTS.len());
}

#[test]
fn short_names_keep_the_columns_that_long_names_give_way_to() {
    // Room that is short for a long name only
    let room = (NAME_LEAST + NAME_MOST) / 2;
    let short = columns_named(Kind::Audio, "Rain", room);
    let long = columns_named(Kind::Audio, &"n".repeat(NAME_MOST), room);
    assert_eq!(short.1, short.0);
    assert!(long.1 < long.0);
}

/// Returns the width that the Target column takes for a file with a name of
/// `length` characters in a list `room` wide, given the room that shows every
/// column whole and leaves a name just [`NAME_MOST`] columns
fn target_width(length: usize, room: impl Fn(f32) -> f32) -> Option<f32> {
    let app = lone(
        Cue::new(CueId(1), "1", Kind::Audio, "n".repeat(NAME_MOST))
            .file("x".repeat(length - 4) + ".wav")
            .build(),
    );
    let layout = Layout::of(app.current());
    let (_, taken) = whole(&app, &layout);
    fitting(&app, &layout, room(taken + NAME_MOST as f32 * char_width()))
        .into_iter()
        .find_map(|(column, width)| (column == Column::Target).then_some(width).flatten())
}

#[test]
fn a_long_file_name_shows_whole_until_a_name_would_lack_room() {
    let columns = |columns: f32| Some(columns * char_width());
    assert_eq!(target_width(40, |enough| enough), columns(40.0));
    // Narrowed to the least, which leaves the name 3 columns over its room
    assert_eq!(
        target_width(40, |enough| enough - char_width()),
        columns(39.0)
    );
}

#[test]
fn the_target_column_grows_into_the_room_that_a_name_leaves_over() {
    let columns = |columns: f32| Some(columns * char_width());
    assert_eq!(
        target_width(100, |enough| enough + 10.0 * char_width()),
        columns(58.0)
    );
    assert_eq!(target_width(100, |enough| enough * 4.0), columns(80.0));
    assert_eq!(target_width(60, |enough| enough * 4.0), columns(60.0));
}

/// Returns the room that `fitting` leaves a name in a list `room` wide whose
/// one cue has a name and a file name of 55 and 65 characters, and whether
/// any column that gives way is still shown
fn name_room(room: f32) -> (f32, bool) {
    let app = lone(
        Cue::new(CueId(1), "1", Kind::Audio, "n".repeat(55))
            .file("x".repeat(61) + ".wav")
            .build(),
    );
    let layout = Layout::of(app.current());
    let kept = fitting(&app, &layout, room);
    let taken: f32 = kept
        .iter()
        .map(|(_, width)| width.unwrap_or(0.0) + CELL_SPACE)
        .sum();
    let yielding = kept
        .iter()
        .any(|(column, _)| Column::YIELDING.contains(column));
    (room - taken, yielding)
}

#[test]
fn a_wider_list_never_gives_a_name_less_than_the_room_that_it_keeps() {
    (600..3000).step_by(10).for_each(|room| {
        let (name, yielding) = name_room(room as f32);
        assert!(
            !yielding || name >= NAME_MOST as f32 * char_width(),
            "{room}: {name}"
        );
    });
}

#[test]
fn a_wider_list_never_gives_a_name_less_room_than_it_needs() {
    let wanted = NAME_MOST as f32 * char_width();
    let mut worse = (600..3000)
        .map(|room| room as f32)
        .filter(|&room| name_room(room + 1.0).0 < name_room(room).0.min(wanted) - 0.01)
        .map(|room| (room, name_room(room).0, name_room(room + 1.0).0));
    assert_eq!(worse.next(), None);
}

#[test]
fn a_name_or_a_message_with_line_breaks_takes_the_room_of_its_one_line() {
    let message = Cue::new(CueId(1), "1", Kind::Network, "Send")
        .message("/scene\n\t\"Act 1\"\r\n2")
        .build();
    [
        (
            Cue::new(CueId(1), "1", Kind::Memo, "two\nlines").build(),
            Column::Name,
            "two lines",
        ),
        (message, Column::Target, "/scene \"Act 1\" 2"),
    ]
    .into_iter()
    .for_each(|(cue, column, text)| {
        let list = CueList::new("Main", vec![cue.clone()]);
        let groups = Layout::of(&list).groups;
        assert_eq!(column.text(&Workspace::default(), &groups, &cue), text);
    });
}
