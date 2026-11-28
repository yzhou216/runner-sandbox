// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Operations that select the text of the text field that has the keyboard
//!
//! Only the name and search fields have an identifier, so the other fields
//! of the inspector cannot be named to `select_all`.  A text field also
//! reports itself as a text input before it reports itself as focusable,
//! which leaves no way to select it on the same visit that finds the focus.
//! So the first operation finds the bounds of the focused widget, and the
//! second selects the text input at those bounds.

use iced::{
    Rectangle, Task,
    advanced::widget::{
        Id, Operation, operate,
        operation::{Focusable, Outcome, TextInput},
    },
};

pub(in crate::front::update) fn select_focused<T: Send + 'static>() -> Task<T> {
    operate(Focused(None)).then(|bounds| operate(SelectAt(bounds)))
}

struct Focused(Option<Rectangle>);

impl Operation<Rectangle> for Focused {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Rectangle>)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        self.0 = self.0.or(state.is_focused().then_some(bounds));
    }

    fn finish(&self) -> Outcome<Rectangle> {
        self.0.map_or(Outcome::None, Outcome::Some)
    }
}

struct SelectAt(Rectangle);

impl<T> Operation<T> for SelectAt {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
        operate(self);
    }

    fn text_input(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn TextInput) {
        if bounds == self.0 {
            state.select_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::front::fixtures::Field;

    /// Runs both operations over the fields, each visited as iced visits a
    /// text field, and tells which ended up selected
    fn selected(focused: [bool; 3]) -> Vec<bool> {
        let mut fields = focused.map(Field::new);
        let at = |row: usize| Rectangle {
            y: row as f32 * 40.0,
            ..Rectangle::default()
        };
        let mut finding = Focused(None);
        fields.iter_mut().enumerate().for_each(|(row, field)| {
            Operation::focusable(&mut finding, None, at(row), field);
        });
        if let Outcome::Some(bounds) = Operation::finish(&finding) {
            let mut selecting = SelectAt(bounds);
            fields.iter_mut().enumerate().for_each(|(row, field)| {
                Operation::<()>::text_input(&mut selecting, None, at(row), field);
            });
        }
        fields.iter().map(|field| field.selected).collect()
    }

    #[test]
    fn only_the_field_that_has_the_focus_is_selected_and_none_when_no_field_has_it() {
        assert_eq!(selected([false, true, false]), [false, true, false]);
        assert_eq!(selected([true, false, false]), [true, false, false]);
        assert_eq!(selected([false; 3]), [false; 3]);
    }
}
