// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fixtures that the tests of several parts of the interface share

use iced::{
    Rectangle,
    advanced::widget::operation::{Focusable, Scrollable, TextInput},
    widget::operation::{AbsoluteOffset, RelativeOffset},
};

use super::App;
use crate::back::{Command, Output, OutputId, Workspace, fresh};

/// Scrollable that tells where it was scrolled to
#[derive(Default)]
pub(super) struct Scrolled(pub(super) Option<f32>);

impl Scrollable for Scrolled {
    fn snap_to(&mut self, _offset: RelativeOffset<Option<f32>>) {}

    fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>) {
        self.0 = offset.y;
    }

    fn scroll_by(
        &mut self,
        _offset: AbsoluteOffset,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
    ) {
    }
}

/// Returns the interface of a show of two outputs, Main and Output 1
pub(super) fn two_outputs() -> App {
    app_of(fresh(vec![]).apply(Command::add_output(1, 1)))
}

pub(super) fn app_of(workspace: Workspace) -> App {
    App {
        workspace,
        ..App::default()
    }
}

/// Returns Main, the first output of the show of `app`
pub(super) fn main_of(app: &App) -> Output {
    app.workspace.output(OutputId(1)).expect("Main").clone()
}

/// Widget that can have the keyboard and select its text, which the tests of
/// operations visit as iced does
#[derive(Default)]
pub(super) struct Field {
    pub(super) focused: bool,
    pub(super) selected: bool,
}

impl Field {
    pub(super) fn new(focused: bool) -> Self {
        Self {
            focused,
            ..Self::default()
        }
    }
}

impl Focusable for Field {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl TextInput for Field {
    fn text(&self) -> &str {
        ""
    }

    fn move_cursor_to_front(&mut self) {}

    fn move_cursor_to_end(&mut self) {}

    fn move_cursor_to(&mut self, _position: usize) {}

    fn select_all(&mut self) {
        self.selected = true;
    }

    fn select_range(&mut self, _start: usize, _end: usize) {}
}
