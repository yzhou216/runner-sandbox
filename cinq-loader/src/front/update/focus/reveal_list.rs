// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scrolling the sidebar to the list that shows
//!
//! The lists lie at the top of a sidebar that scrolls when the window is low,
//! so a switch of list, by a key or by a jump to a cue of another list, can
//! leave the filled radio out of view.  The operation finds the entry that the
//! sidebar marks as current and scrolls to show it, as little as it can.

use iced::{
    Rectangle, Task, Vector,
    advanced::widget::{
        Id, Operation, operate,
        operation::{Outcome, Scrollable},
    },
};

use super::super::rows::scroll::scrolled_to;
use crate::front::{App, stepping::reveal, view};

impl App {
    /// Returns what the sidebar shows as current, which is a list or the
    /// outputs
    pub(in crate::front::update) fn shown_list(&self) -> (usize, bool) {
        (self.list, self.outputs_shown)
    }
}

pub(in crate::front::update) fn reveal_current_list<T: Send + 'static>() -> Task<T> {
    operate(Current::default()).discard()
}

#[derive(Default)]
struct Current {
    /// Top of the content, how far it has scrolled, and the height in view
    list: Option<(f32, f32, f32)>,
    entry: Option<Rectangle>,
}

impl Operation for Current {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
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
        if id == Some(&Id::new(view::SIDEBAR_SCROLL)) {
            self.list = Some((content_bounds.y, translation.y, bounds.height));
        }
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&Id::new(view::CURRENT_LIST)) {
            self.entry = Some(bounds);
        }
    }

    fn finish(&self) -> Outcome<()> {
        let (Some((origin, scrolled, height)), Some(entry)) = (self.list, self.entry) else {
            return Outcome::None;
        };
        let top = entry.y - origin;
        // Gap of the column on each side keeps the whole radio in sight
        let (above, below) = (top - view::SPACING, top + entry.height + view::SPACING);
        scrolled_to(view::SIDEBAR_SCROLL, reveal(above, below, scrolled, height))
    }
}

#[cfg(test)]
mod tests;
