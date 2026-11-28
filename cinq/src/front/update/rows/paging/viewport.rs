// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Operation that measures a scrollable as it is laid out

use iced::{
    Rectangle, Vector,
    advanced::widget::{
        Id,
        operation::{Operation, Outcome, Scrollable},
    },
};

/// How a scrollable sits in its view
#[derive(Debug, Clone, Copy)]
pub struct Shape {
    /// Height of the part in view
    pub view: f32,
    /// Height of all the content
    pub content: f32,
    /// How far the content has scrolled
    pub scrolled: f32,
}

impl Shape {
    /// Returns the first and the last row, counted from zero, that are in
    /// view in a list of `rows` rows, or `None` for no rows.  A row is in
    /// view when more than half of it shows, so that the scroll offset, which
    /// the list reports in whole pixels, never decides the rows that a key
    /// moves between.  The rows are as tall as the average, since the rows
    /// that a long name wraps are taller than the rest.  Not even one row
    /// fitting leaves the first row as both.
    pub(in crate::front::update) fn rows_in_view(self, rows: usize) -> Option<(usize, usize)> {
        let last_row = rows.checked_sub(1)?;
        let row_height = self.content / rows as f32;
        let first = ((self.scrolled / row_height + 0.5).floor() as usize).min(last_row);
        let bottom = ((self.scrolled + self.view) / row_height - 0.5).ceil() as usize;
        Some((first, bottom.saturating_sub(1).clamp(first, last_row)))
    }
}

/// Finds the shape of a scrollable by its identity
pub(super) struct Viewport {
    id: &'static str,
    found: Option<Shape>,
}

impl Viewport {
    pub(super) fn of(id: &'static str) -> Self {
        Self { id, found: None }
    }
}

impl Operation<Shape> for Viewport {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Shape>)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        if id == Some(&Id::new(self.id)) {
            self.found = Some(Shape {
                view: bounds.height,
                content: content_bounds.height,
                scrolled: translation.y,
            });
        }
    }

    fn finish(&self) -> Outcome<Shape> {
        self.found.map_or(Outcome::None, Outcome::Some)
    }
}
