// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Menu of a drop-down list, which opens below the list or above it,
//! wherever there is more room, and scrolls when the options do not fit
//!
//! The options sit in iced's scrollable, so the menu scrolls and shows its
//! scroll bar as iced's own menu does.  The mouse highlights the option
//! under it and picks the option it presses, and the keys move the
//! highlight, which the menu scrolls into view.  A click beside the menu
//! closes it and acts as it would with the menu closed, apart from a click
//! on the list, which only closes the menu.
//!
//! iced hands each batch of events first to the menu that is open when the
//! batch begins, then to the widgets under it, so a menu that closes
//! partway through a batch leaves the rest of the batch to them.

mod overlay;
mod rows;
#[cfg(test)]
mod tests;

use std::cell::Cell;

use iced::{
    advanced::Shell,
    keyboard::{Key, key::Named},
};

use super::Dropdown;
use crate::front::{
    Message,
    keymap::{Action, Keymap, Press},
    step,
};
pub(super) use overlay::Menu;

/// State of the menu, which the list keeps in cells, since the list and its
/// open menu both act on it
#[derive(Default)]
pub(super) struct Shared {
    open: Cell<bool>,
    /// Option under the highlight, which the mouse moves as well as the keys,
    /// or none while it stays on the selected option.  The list finds the
    /// selected option afresh each time it is built, so that a highlight
    /// left on it follows it when the options change under the open menu.
    highlight: Cell<Option<usize>>,
    /// Whether the menu has yet to scroll the highlight into view
    reveal: Cell<bool>,
}

impl Shared {
    pub(super) fn is_open(&self) -> bool {
        self.open.get()
    }
}

/// A drop-down list with the state of its menu, through which the list and
/// the menu act on the menu
#[derive(Clone, Copy)]
pub(super) struct Choice<'b> {
    dropdown: &'b Dropdown<'b>,
    shared: &'b Shared,
}

impl<'b> Choice<'b> {
    pub(super) fn new(dropdown: &'b Dropdown<'b>, shared: &'b Shared) -> Self {
        Self { dropdown, shared }
    }

    pub(super) fn highlighted(&self) -> Option<usize> {
        self.shared.highlight.get().or(self.dropdown.chosen)
    }

    /// Opens the menu with the highlight on the selected option, unless
    /// there is no option to choose
    pub(super) fn open(&self, shell: &mut Shell<'_, Message>) {
        if !self.dropdown.names.is_empty() {
            self.shared.open.set(true);
            self.shared.highlight.set(None);
            self.shared.reveal.set(true);
        }
        if let Some(message) = &self.dropdown.on_open {
            shell.publish(message.clone());
        }
        shell.capture_event();
        shell.request_redraw();
    }

    pub(super) fn press(&self, key: &Key, press: Option<Press>, shell: &mut Shell<'_, Message>) {
        match reply(self.dropdown.keymap, key, press) {
            Reply::Step(offset) => {
                let row = step(self.highlighted(), offset, self.dropdown.names.len());
                self.shared.highlight.set(row);
                self.shared.reveal.set(true);
                shell.capture_event();
            }
            Reply::Pick => self.pick(self.highlighted(), shell),
            Reply::Close => self.shared.open.set(false),
            Reply::Pass => {}
            Reply::Keep => shell.capture_event(),
        }
        shell.request_redraw();
    }

    /// Closes the menu on a click beside it, which goes on to act on what it
    /// lands on, unless it lands on the list, whose menu it would open again
    pub(super) fn click_beside(&self, on_list: bool, shell: &mut Shell<'_, Message>) {
        self.shared.open.set(false);
        if on_list {
            shell.capture_event();
        }
        shell.request_redraw();
    }

    fn pick(&self, row: Option<usize>, shell: &mut Shell<'_, Message>) {
        if let Some(message) = row.and_then(|row| (self.dropdown.pick)(row)) {
            shell.publish(message);
        }
        self.shared.open.set(false);
        shell.capture_event();
    }
}

/// What a key does while the menu is open
#[derive(Debug, PartialEq)]
enum Reply {
    /// Moves the highlight by a number of options, downwards when positive
    Step(isize),
    Pick,
    /// Lets the key act and closes the menu, as the keys of GO and Panic do
    Close,
    /// Lets the key act and leaves the menu open, as the key of Help does:
    /// the page hides the menu, which is open still when the page closes
    Pass,
    /// Keeps the key from acting on anything behind the menu
    Keep,
}

/// Decides what a key does, given the press it makes when the key bindings
/// can name it
fn reply(keymap: &Keymap, key: &Key, press: Option<Press>) -> Reply {
    match (press.and_then(|press| keymap.alone(press)), key) {
        (Some(Action::Next), _) => Reply::Step(1),
        (Some(Action::Previous), _) => Reply::Step(-1),
        (Some(action), _) if action.acts_at_once() => Reply::Close,
        (Some(Action::Help), _) => Reply::Pass,
        (_, Key::Named(Named::Enter)) => Reply::Pick,
        _ => Reply::Keep,
    }
}
