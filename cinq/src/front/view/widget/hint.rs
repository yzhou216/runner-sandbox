// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Places that a click acts on, which hint mode labels
//!
//! [`hinted`] wraps a widget that a click acts on and tells every operation
//! about it, with the message that the click would send.  One walk when hint
//! mode starts collects the places that the operator can see, in the order that
//! they lie in the tree, which gives the labels their order, and leaves out
//! those inside a [`hide`](super::hide()), and those whose top left corner, which
//! the label lies level with, is outside the window or outside a scrollable
//! around them.  The app keeps the labels, and [`layer()`] draws them over the
//! window.  A text field or a drop-down list takes the keyboard when [`reach`]
//! finds it again by its bounds.

mod layer;
mod visit;

#[cfg(test)]
mod tests;

use iced::{
    Element, Rectangle,
    advanced::{layout::Layout, widget::Operation},
};

use super::wrap::{Intercept, wrap};
use crate::front::Message;
pub use layer::{corners, layer, width};
pub use visit::{places, reach};

/// What choosing a place by its label does
#[derive(Debug, Clone)]
pub enum Act {
    /// Sends the message that a click on the place sends
    Send(Message),
    /// Has the place take the keyboard, which only the widget can do
    Reach,
}

impl From<Message> for Act {
    fn from(message: Message) -> Self {
        Act::Send(message)
    }
}

/// Place that a click acts on, as the walk found it
#[derive(Debug, Clone)]
pub struct Site {
    /// Bounds in the window, which the label goes left of, at the same height
    pub shown: Rectangle,
    /// Bounds in the layout, as the widget has them, which find it again
    pub at: Rectangle,
    pub act: Act,
}

/// What a place that a click acts on tells operations about itself
pub struct Target(pub Act);

impl Intercept for Target {
    fn operate(&self, layout: Layout<'_>, operation: &mut dyn Operation) {
        operation.custom(None, layout.bounds(), &mut Target(self.0.clone()));
    }
}

/// Marks the content as a place that a click acts on, which `act` does when
/// its label is typed.  A [`Message`] stands for an act that sends it.
pub fn hinted<'a>(
    content: impl Into<Element<'a, Message>>,
    act: impl Into<Act>,
) -> Element<'a, Message> {
    wrap(content, Target(act.into()))
}

/// Marks the content as a text field, which its label gives the keyboard
pub fn reached<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    hinted(content, Act::Reach)
}

/// Marks the content as the window, which the places in it must lie inside
pub fn window<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    wrap(content, Window)
}

/// What the content of a window tells operations about its bounds
struct Window;

impl Intercept for Window {
    fn operate(&self, layout: Layout<'_>, operation: &mut dyn Operation) {
        operation.custom(None, layout.bounds(), &mut Window);
    }
}

/// What content that is out of sight tells operations as it begins, and as it
/// ends, so that they leave out the places in it
pub struct Hidden(pub bool);
