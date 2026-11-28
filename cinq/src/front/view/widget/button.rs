// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Buttons that act as soon as they are pressed
//!
//! iced's own buttons send their message only when the mouse button comes
//! back up, while every other control acts on the press, so an operator
//! holding GO would wait for the cue until letting go.  These buttons keep
//! the look of iced's, but they send their message on the press.  The press
//! never reaches the iced button inside, so it cannot send the message again
//! on release, and a button held down keeps the look of one hovered.  A button
//! that is made [`reachable`](Button::reachable) is also a stop of the Tab
//! order, which Enter presses.

use iced::{
    Element, Event, Length, Theme,
    advanced::{Shell, layout::Layout},
    mouse,
    widget::button::{Status, Style},
};

use super::{
    focus::Focused,
    hinted, pressed,
    wrap::{Intercept, wrap},
};
use crate::front::{Message, keymap::Keymap, view::theme};

pub struct Button<'a> {
    /// iced's button, which draws this one but never sees a press
    look: iced::widget::Button<'a, Message>,
    on_press: Option<Message>,
    /// Keymap of a button that Tab reaches and Enter presses
    keymap: Option<&'a Keymap>,
    /// Whether Enter gives the keyboard back to the cue list after it presses
    /// the button
    leaves: bool,
    /// Whether hint mode labels the button
    hinted: bool,
}

/// Returns a disabled button that shows `content`, until it has a message
pub fn button<'a>(content: impl Into<Element<'a, Message>>) -> Button<'a> {
    Button {
        look: iced::widget::button(content).style(theme::button),
        on_press: None,
        keymap: None,
        leaves: false,
        hinted: true,
    }
}

impl Button<'_> {
    pub fn on_press(self, message: Message) -> Self {
        self.on_press_maybe(Some(message))
    }

    pub fn on_press_maybe(self, message: Option<Message>) -> Self {
        Self {
            look: self.look.on_press_maybe(message.clone()),
            on_press: message,
            ..self
        }
    }

    pub fn width(self, width: impl Into<Length>) -> Self {
        Self {
            look: self.look.width(width),
            ..self
        }
    }

    pub fn style(self, style: fn(&Theme, Status) -> Style) -> Self {
        Self {
            look: self.look.style(style),
            ..self
        }
    }
}

impl<'a> Button<'a> {
    /// Leaves the button out of hint mode, for a button whose key acts from
    /// everywhere, so that its label would add no reach
    pub fn unhinted(self) -> Self {
        Self {
            hinted: false,
            ..self
        }
    }

    /// Returns the message that hint mode sends for the button, unless it is
    /// disabled or left out of hint mode
    pub(in crate::front::view) fn place(&self) -> Option<&Message> {
        self.on_press.as_ref().filter(|_| self.hinted)
    }

    /// Makes the button a stop of the Tab order, which Enter presses, unless
    /// it has no message, as it has not while show mode locks it
    pub fn reachable(self, keymap: &'a Keymap) -> Self {
        Self {
            keymap: Some(keymap),
            ..self
        }
    }

    /// Makes the button reachable as [`Self::reachable`] does, and gives the
    /// keyboard back to the cue list after Enter presses it, for a button whose
    /// press changes the rows under it, since iced would otherwise pass the
    /// keyboard to the same button of the row that slides into its place
    pub fn reachable_and_leave(self, keymap: &'a Keymap) -> Self {
        Self {
            leaves: true,
            ..self.reachable(keymap)
        }
    }
}

impl<'a> From<Button<'a>> for Element<'a, Message> {
    fn from(button: Button<'a>) -> Self {
        let place = button.place().cloned();
        let look = wrap(button.look, OnPress(button.on_press.clone()));
        let reached = match (button.keymap, button.on_press) {
            (Some(keymap), Some(press)) => {
                Element::new(Focused::button(look, press, button.leaves, keymap))
            }
            _ => look,
        };
        match place {
            Some(press) => hinted(reached, press),
            None => reached,
        }
    }
}

struct OnPress(Option<Message>);

impl Intercept for OnPress {
    fn before(
        &self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let press = pressed(event);
        if press
            && !shell.is_event_captured()
            && cursor.is_over(layout.bounds())
            && let Some(message) = &self.0
        {
            shell.publish(message.clone());
            shell.capture_event();
        }
        !press
    }
}
