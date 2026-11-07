// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! State that iced keeps for a drop-down list from one frame to the next,
//! which keeps the labels that set the width of the list, and through
//! which the keys that move between fields focus the list

use iced::{
    Renderer,
    advanced::{
        text::paragraph,
        widget::{Tree, operation::Focusable},
    },
};

use super::{label::label, menu::Shared};
use crate::front::view::Paragraph;

pub(super) struct State {
    pub(super) shared: Shared,
    /// Whether the list has the keyboard, which it has only until its menu
    /// opens
    pub(super) focused: bool,
    /// Whether the cursor was over the list in the last frame, so that the
    /// list redraws when its look changes
    pub(super) over: bool,
    /// Labels of the options and the placeholder, which set the width
    pub(super) labels: Vec<paragraph::Plain<Paragraph>>,
    pub(super) placeholder: paragraph::Plain<Paragraph>,
    /// State of the widgets of the menu, which outlives each opening
    pub(super) menu: Tree,
}

impl State {
    pub(super) fn widest<T: ToString>(
        &mut self,
        renderer: &Renderer,
        options: &[T],
        placeholder: Option<&str>,
    ) -> f32 {
        let width = |plain: &mut paragraph::Plain<Paragraph>, name: &str| {
            plain.update(label(renderer, name, f32::INFINITY));
            plain.min_width()
        };
        self.labels.resize_with(options.len(), Default::default);
        self.labels
            .iter_mut()
            .zip(options)
            .map(|(plain, option)| width(plain, &option.to_string()))
            .chain(placeholder.map(|placeholder| width(&mut self.placeholder, placeholder)))
            .fold(0.0, f32::max)
    }
}

impl Focusable for State {
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
