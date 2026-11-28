// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the columns of the outputs view, which give way to the device

use super::{Column, DEVICE_ROOM, device_room, fitting};
use crate::{
    back::{Command, Device, Fault, OutputEdit, OutputId, fresh},
    front::{
        App, Problems,
        fixtures::{app_of, in_one_run, wide},
        view::char_width,
    },
};

/// Returns an app whose Main has a long name and a state that tells that
/// its channels lie past those of its device
fn app() -> App {
    let name = "Far away speakers of the balcony".to_owned();
    App {
        workspace: fresh(Vec::new())
            .apply(Command::EditOutput(OutputId(1), OutputEdit::Name(name))),
        problems: Problems {
            audio: vec![(OutputId(1), Fault::Channels(64))],
            ..Problems::default()
        },
        ..App::default()
    }
}

/// Returns an app whose Main plays on a device named `name`
fn on_device(name: &str) -> App {
    let device = Device::Named {
        id: name.to_owned(),
        name: name.to_owned(),
    };
    app_of(fresh(Vec::new()).apply(Command::EditOutput(OutputId(1), OutputEdit::Device(device))))
}

/// Returns whether the table of `room` columns keeps the New cues column
fn shows_new_cues(app: &App, room: f32) -> bool {
    fitting(app, wide(room))
        .iter()
        .any(|(column, _)| matches!(column, Column::NewCues))
}

#[test]
fn device_keeps_its_room_while_the_other_columns_give_way() {
    let app = app();
    [69.0, 73.0, 83.0, 125.0]
        .into_iter()
        .map(wide)
        .for_each(|room| {
            let columns = fitting(&app, room);
            assert!(
                device_room(&columns, room) >= DEVICE_ROOM * char_width(),
                "{room}"
            );
        });
}

#[test]
fn the_new_cues_column_shows_in_a_wide_table_and_goes_from_a_narrow_one() {
    assert!(shows_new_cues(&app(), 166.0));
    assert!(!shows_new_cues(&app(), 69.0));
}

#[test]
fn new_cues_stays_only_while_the_device_shows_on_one_line() {
    // Device name of 23 columns, which a table narrower than the whole of its
    // columns wraps
    let app = on_device("Dante Virtual Soundcard");
    let kept = in_one_run(40..200, |room| shows_new_cues(&app, room));
    assert!(!kept.is_empty(), "a wide table keeps the column");
    assert!(shows_new_cues(&app, 199.0) && !shows_new_cues(&app, 40.0));
    let narrowest = kept[0];
    assert!(!shows_new_cues(&app, narrowest - 1.0));
    let columns = fitting(&app, wide(narrowest));
    assert!(device_room(&columns, wide(narrowest)) >= 23.0 * char_width());
}

#[test]
fn a_device_name_too_long_for_any_table_does_not_drive_new_cues_away() {
    assert!(shows_new_cues(&on_device(&"x".repeat(60)), 166.0));
}

/// Returns whether the table of `room` columns shows the `column`
fn shows(app: &App, column: Column, room: f32) -> bool {
    fitting(app, wide(room))
        .iter()
        .any(|(shown, _)| *shown == column)
}

#[test]
fn the_state_and_channels_columns_go_last_so_that_a_narrow_window_keeps_the_device() {
    let app = app();
    [Column::State, Column::Channels]
        .into_iter()
        .for_each(|column| {
            in_one_run(30..200, |room| shows(&app, column, room));
            assert!(!shows(&app, column, 30.0) && shows(&app, column, 199.0));
        });
    // State goes before Channels does
    let narrowest = |column| (30..200).find(|&room| shows(&app, column, room as f32));
    assert!(narrowest(Column::State) > narrowest(Column::Channels));
}

#[test]
fn the_device_keeps_a_name_of_two_words_whole_in_a_window_of_512_pixels() {
    // The table has about 35 columns there, beside the sidebar
    let app = on_device("Default device");
    let room = 35.0;
    let columns = fitting(&app, wide(room));
    assert!(device_room(&columns, wide(room)) >= 14.0 * char_width());
}

#[test]
fn a_name_with_a_line_break_shows_on_one_line() {
    let app = app_of(fresh(Vec::new()).apply(Command::EditOutput(
        OutputId(1),
        OutputEdit::Name("House\n\tLeft".to_owned()),
    )));
    let output = &app.workspace.outputs()[0];
    assert_eq!(Column::Name.text(&app, 0, output), "House Left");
}
