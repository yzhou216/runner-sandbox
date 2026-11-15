// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the columns of the outputs view, which give way to the device

use super::{Column, DEVICE_ROOM, device_room, fitting};
use crate::{
    back::{Command, CueList, Fault, OutputEdit, OutputId, Workspace},
    front::{App, view::char_width},
};

/// Returns an app whose Main has a long name and a state that tells that
/// its channels lie past those of its device
fn app() -> App {
    let name = "Far away speakers of the balcony".to_owned();
    App {
        workspace: Workspace::new(vec![CueList::new("Main", Vec::new())])
            .apply(Command::EditOutput(OutputId(1), OutputEdit::Name(name))),
        audio_problems: vec![(OutputId(1), Fault::Channels(64))],
        ..App::default()
    }
}

#[test]
fn device_keeps_its_room_while_the_other_columns_give_way() {
    let app = app();
    [660.0, 700.0, 800.0, 1200.0].into_iter().for_each(|room| {
        let columns = fitting(&app, room);
        assert!(
            device_room(&columns, room) >= DEVICE_ROOM * char_width(),
            "{room}"
        );
    });
}

#[test]
fn nothing_gives_way_where_everything_fits() {
    let columns = fitting(&app(), 1600.0);
    assert!(
        columns
            .iter()
            .any(|(column, _)| matches!(column, Column::NewCues))
    );
    let narrow = fitting(&app(), 660.0);
    assert!(
        narrow
            .iter()
            .all(|(column, _)| !matches!(column, Column::NewCues))
    );
}
