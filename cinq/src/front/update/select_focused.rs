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

pub(super) fn select_focused<T: Send + 'static>() -> Task<T> {
    operate(Focused(None)).then(|bounds| operate(SelectAt(bounds)))
}

/// Finds the bounds of the widget that has the focus
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

/// Selects the text of the text input at the given bounds
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

    struct Field {
        bounds: Rectangle,
        focused: bool,
        selected: bool,
    }

    impl Focusable for Field {
        fn is_focused(&self) -> bool {
            self.focused
        }

        fn focus(&mut self) {}

        fn unfocus(&mut self) {}
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

    /// Runs both operations over the fields, each visited as iced visits a
    /// text field, and tells which ended up selected
    fn selected(focused: [bool; 3]) -> Vec<bool> {
        let mut fields = focused.map(|focused| Field {
            bounds: Rectangle::default(),
            focused,
            selected: false,
        });
        fields.iter_mut().enumerate().for_each(|(row, field)| {
            field.bounds.y = row as f32 * 40.0;
        });
        let mut finding = Focused(None);
        fields.iter_mut().for_each(|field| {
            Operation::focusable(&mut finding, None, field.bounds, field);
        });
        if let Outcome::Some(bounds) = Operation::finish(&finding) {
            let mut selecting = SelectAt(bounds);
            fields.iter_mut().for_each(|field| {
                Operation::<()>::text_input(&mut selecting, None, field.bounds, field);
            });
        }
        fields.iter().map(|field| field.selected).collect()
    }

    #[test]
    fn only_the_field_that_has_the_focus_is_selected() {
        assert_eq!(selected([false, true, false]), [false, true, false]);
        assert_eq!(selected([true, false, false]), [true, false, false]);
    }

    #[test]
    fn no_field_is_selected_when_none_has_the_focus() {
        assert_eq!(selected([false; 3]), [false; 3]);
    }
}
