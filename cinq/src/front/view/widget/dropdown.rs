// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drop-down lists whose menus the keys move through
//!
//! iced's pick list takes no keys, so keys pressed over its open menu act
//! on the cue list behind it.  These lists look like iced's pick list but
//! open a [`menu`] of their own, in which the keys of [`Action::Next`] and
//! [`Action::Previous`] move the highlight, which the menu keeps in view,
//! and Enter or the key of [`Action::Pick`] picks the highlighted option,
//! while the key of [`Action::CloseMenu`] closes the menu without a pick.
//! PageDown, PageUp, Home, and End move the highlight a page or to either
//! end, as in Vim's popup menu.  GO and Panic act as everywhere and close the
//! menu, so that the show is in view, and the menu keeps every other key
//! until it closes.  The interface hears of each opening and closing, so that
//! its status bar hints the keys of an open menu.
//!
//! The keys that move between fields move to these lists as well, which
//! then show the border of a focused field, and Enter opens the menu, which
//! takes the keys from there, unless Enter is bound to GO or Panic, which
//! act as everywhere.  The list hands the keyboard to its menu as it
//! opens, so that the keys act on the cue list again once the menu closes,
//! as they do after Enter leaves a text field.  A pick or a close by key hands
//! the keyboard back to the list, so that the key that moves on goes to the
//! field after it.
//!
//! A list with less room than its longest name cuts the name it shows
//! short, and its menu grows as wide as that name, so that a list in the
//! narrow sidebar can hold the long names of audio devices.
//!
//! [`Action::Next`]: crate::front::keymap::Action::Next
//! [`Action::Previous`]: crate::front::keymap::Action::Previous
//! [`Action::Pick`]: crate::front::keymap::Action::Pick
//! [`Action::CloseMenu`]: crate::front::keymap::Action::CloseMenu

mod menu;
mod shorten;
mod state;
mod widget;

use iced::{Color, Element, Padding, widget::button};
use unicode_width::UnicodeWidthStr;

use crate::front::{Message, keymap::Keymap, view::char_width};

/// Padding of the list and of each option of its menu, as iced's pick list
/// pads them
const PADDING: Padding = button::DEFAULT_PADDING;

pub struct Dropdown<'a> {
    names: Vec<String>,
    /// Index of the selected option
    chosen: Option<usize>,
    /// Makes the message of the option at an index, if there is one
    pick: Box<dyn Fn(usize) -> Option<Message> + 'a>,
    /// Message that each opening of the menu sends, such as one that asks
    /// for the options afresh
    on_open: Option<Message>,
    /// Makes the message that tells which option the highlight of the open
    /// menu rests on, or none once the menu closes
    on_highlight: Option<Box<dyn Fn(Option<usize>) -> Message + 'a>>,
    /// Whether the list wears the colors of the highlight of a menu, which
    /// tells that an option of another menu depends on this list
    lit: bool,
    placeholder: Option<String>,
    /// Color of the placeholder, where it tells more than that nothing is
    /// chosen, as that the device chosen is not there
    placeholder_color: Option<Color>,
    keymap: &'a Keymap,
}

pub fn dropdown<'a, T: ToString + PartialEq + Clone + 'a>(
    options: impl Into<Vec<T>>,
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
    keymap: &'a Keymap,
) -> Dropdown<'a> {
    let options = options.into();
    Dropdown {
        names: options.iter().map(T::to_string).collect(),
        chosen: options
            .iter()
            .position(|option| Some(option) == selected.as_ref()),
        pick: Box::new(move |row| options.get(row).cloned().map(&on_select)),
        on_open: None,
        on_highlight: None,
        lit: false,
        placeholder: None,
        placeholder_color: None,
        keymap,
    }
}

impl<'a> Dropdown<'a> {
    pub fn placeholder(self, placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: Some(placeholder.into()),
            ..self
        }
    }

    pub fn placeholder_color(self, color: Color) -> Self {
        Self {
            placeholder_color: Some(color),
            ..self
        }
    }

    pub fn on_open(self, message: Message) -> Self {
        Self {
            on_open: Some(message),
            ..self
        }
    }

    /// Has the open menu tell, with the message that `tell` makes, which
    /// option its highlight rests on whenever that changes, and that it rests
    /// on none once the menu closes
    pub fn on_highlight(self, tell: impl Fn(Option<usize>) -> Message + 'a) -> Self {
        Self {
            on_highlight: Some(Box::new(tell)),
            ..self
        }
    }

    /// Draws the list in the colors of the highlight of a menu where `lit`
    pub fn lit(self, lit: bool) -> Self {
        Self { lit, ..self }
    }

    /// Returns the width of the longest name, the placeholder among them,
    /// by the columns of the font, which is monospace
    fn widest(&self) -> f32 {
        let columns = (self.names.iter().map(String::as_str))
            .chain(self.placeholder.as_deref())
            .map(UnicodeWidthStr::width)
            .max()
            .unwrap_or(0);
        columns as f32 * char_width()
    }
}

impl<'a> From<Dropdown<'a>> for Element<'a, Message> {
    fn from(dropdown: Dropdown<'a>) -> Self {
        Element::new(dropdown)
    }
}

/// Returns the width of a list that shows `columns` columns of a name, with
/// its padding and its arrow
pub fn width(columns: f32) -> f32 {
    columns * char_width() + PADDING.x() + widget::ARROW
}
