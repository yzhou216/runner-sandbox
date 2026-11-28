// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving the keyboard to the next or previous field, with the inspector first
//!
//! The widgets come in the order of the window, which puts the toolbar and the
//! sidebar before the inspector, but the settings of what is selected are what
//! the operator comes for.  The order here puts the fields of the inspector
//! first, then the rest in the order of the window, and goes round at its
//! ends, where it stops once with no field holding the keyboard, so that a
//! checkbox or a closed list never keeps it for good.  Two operations do it,
//! as iced's own do: the first reads the fields, and the second gives the
//! keyboard to the one that the order chose.

use iced::{
    Rectangle, Task,
    advanced::widget::{
        Id, Operation, operate,
        operation::{Focusable, Outcome, focusable},
    },
};

use crate::front::view;

/// Field of the window, in the order of the window
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slot {
    inspector: bool,
    focused: bool,
}

pub(in crate::front::update) fn unfocus<T: Send + 'static>() -> Task<T> {
    operate(focusable::unfocus())
}

/// Moves the keyboard to the field after the one that has it, or before it
/// when `backward`, or to the first or last where none has it, and takes it
/// from every field when it moves past the last or the first
pub(in crate::front::update) fn move_focus<T: Send + 'static>(backward: bool) -> Task<T> {
    operate(Reading::new(view::INSPECTOR)).then(move |slots| match chosen(&slots, backward) {
        Some(target) => operate::<T>(Giving { visited: 0, target }),
        None => Task::none(),
    })
}

/// Returns the place in the order of the window of the field that comes next
/// in the order of this module, where the fields of the inspector come first,
/// or the count of the fields where the next stop is no field, which gives the
/// keys back to the list
fn chosen(slots: &[Slot], backward: bool) -> Option<usize> {
    let order: Vec<usize> = (0..slots.len())
        .filter(|&at| slots[at].inspector)
        .chain((0..slots.len()).filter(|&at| !slots[at].inspector))
        .collect();
    // The stops are the fields and, after the last, no field, where the
    // keyboard stands when none has it
    let stops = order.len() + 1;
    if order.is_empty() {
        return None;
    }
    let here = order
        .iter()
        .position(|&at| slots[at].focused)
        .unwrap_or(order.len());
    let next = (here + if backward { stops - 1 } else { 1 }) % stops;
    Some(order.get(next).copied().unwrap_or(slots.len()))
}

/// Reads the fields of the window, noting those within the inspector
struct Reading {
    inspector: Id,
    /// Whether the next traversal enters the inspector
    entering: bool,
    /// Whether the widgets visited now lie within it
    inside: bool,
    slots: Vec<Slot>,
}

impl Reading {
    fn new(inspector: &'static str) -> Self {
        Self {
            inspector: Id::new(inspector),
            entering: false,
            inside: false,
            slots: Vec::new(),
        }
    }
}

impl Operation<Vec<Slot>> for Reading {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Vec<Slot>>)) {
        let outside = self.inside;
        self.inside |= std::mem::take(&mut self.entering);
        operate(self);
        self.inside = outside;
    }

    fn container(&mut self, id: Option<&Id>, _bounds: Rectangle) {
        self.entering = id == Some(&self.inspector);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        self.slots.push(Slot {
            inspector: self.inside,
            focused: state.is_focused(),
        });
    }

    fn finish(&self) -> Outcome<Vec<Slot>> {
        Outcome::Some(self.slots.clone())
    }
}

/// Focuses the field at a place in the order of the window, which the
/// count of the fields visited so far finds, and takes the keyboard from
/// the rest
struct Giving {
    visited: usize,
    target: usize,
}

impl<T> Operation<T> for Giving {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if self.visited == self.target {
            state.focus();
        } else {
            state.unfocus();
        }
        self.visited += 1;
    }
}

#[cfg(test)]
mod tests;
