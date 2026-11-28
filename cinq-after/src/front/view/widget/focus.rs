// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a checkbox and a button share when Tab reaches them and Enter acts
//!
//! Enter belongs to the focused widget unless a key that acts at once is bound
//! to it, and the key bindings never see it there, where it would put a cue on
//! standby.

#[cfg(test)]
mod tests;
mod widget;

use iced::{
    Border, Color, Event, Rectangle, Renderer, Theme,
    advanced::{Renderer as _, Shell, renderer, widget::operation::Focusable},
    keyboard::{Key, key::Named},
    widget::text_input,
};

use super::{key_pressed, pressed};
use crate::front::{
    Focus, Message,
    keymap::{KeyPress, Keymap},
    view::theme,
};
pub use widget::Focused;

const RADIUS: f32 = 2.0;

pub struct State {
    /// Whether the widget has the keyboard, which any click takes away, as it
    /// does from a text field
    pub focused: bool,
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

/// Draws the border of a focused text field around `bounds`
pub fn mark(renderer: &mut Renderer, theme: &Theme, bounds: Rectangle) {
    let focused = text_input::Status::Focused { is_hovered: false };
    let border = Border {
        color: text_input::default(theme, focused).border.color,
        width: theme::FOCUS_WIDTH,
        radius: RADIUS.into(),
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: bounds.expand(theme::FOCUS_WIDTH),
            border,
            ..renderer::Quad::default()
        },
        Color::TRANSPARENT,
    );
}

/// Handles the keyboard events of a widget that may have the keyboard, and
/// returns whether it took the event.  A press of the left button of the
/// mouse or a touch takes the keyboard from the widget, as a click does from
/// a text field.  The widget takes Enter alone, unless a key that acts at once
/// is bound to it, and keeps the event from the widgets after it.  Enter sends
/// `act` once however long it is held, and then gives the keyboard back to the
/// cue list where `leaves`.
pub fn keyed(
    state: &mut State,
    event: &Event,
    keymap: &Keymap,
    act: Option<&Message>,
    leaves: bool,
    shell: &mut Shell<'_, Message>,
) -> bool {
    if pressed(event) && state.focused {
        state.focused = false;
        shell.request_redraw();
    }
    key_pressed(event).is_some_and(|KeyPress { key, press, repeat }| {
        let at_once = press.is_some_and(|press| keymap.acts_at_once(press));
        let taken = state.focused && !at_once && *key == Key::Named(Named::Enter);
        if taken {
            shell.capture_event();
            if let (false, Some(act)) = (repeat, act) {
                shell.publish(act.clone());
                if leaves {
                    shell.publish(Message::Focus(Focus::Cues));
                }
            }
        }
        taken
    })
}
