// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drop-down list as an iced widget

mod draw;

use iced::{
    Event, Length, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Shell,
        layout::{self, Layout},
        mouse, overlay, renderer,
        widget::{
            Operation, Widget,
            tree::{self, Tree},
        },
    },
    keyboard::{Key, key::Named},
    window,
};

use super::{
    Dropdown, PADDING,
    menu::{Choice, Menu},
    state::State,
};
use crate::front::{
    Message,
    keymap::KeyPress,
    view::{
        TEXT_SIZE, line_height,
        widget::{Act, Target, key_pressed, pointer_over, pressed},
    },
};

/// Room for the arrow, which is about as wide as the text is high
pub(super) const ARROW: f32 = TEXT_SIZE.0 + PADDING.left;

impl Widget<Message, Theme, Renderer> for Dropdown<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    /// Makes the list as wide as its widest label, as iced's pick list is,
    /// so that its width stays put whichever option is chosen, or as wide
    /// as its room, where a longer label is cut short
    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let intrinsic = Size::new(self.widest() + ARROW, line_height());
        layout::Node::new(
            limits
                .shrink(PADDING)
                .resolve(Length::Shrink, Length::Shrink, intrinsic)
                .expand(PADDING),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.drawn(tree, renderer, theme, layout, cursor, viewport);
    }

    /// Opens the menu on a click, or on Enter while the list has the
    /// keyboard, which any click takes away, as it does from a text field.
    /// Once open, the menu takes the keys and clicks itself, apart from the
    /// rest of the batch of events in which it opens, since iced hands that
    /// out before it has the menu, and the list acts on the menu's
    /// [`Choice`] for those.
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let over = cursor.is_over(layout.bounds());
        // Pick or a close by key leaves the keyboard on the list, so that
        // Tab goes on to the next field from it
        if !state.shared.is_open() && state.shared.take_returning() {
            state.focused = true;
            shell.request_redraw();
        }
        if pressed(event) && state.focused {
            state.focused = false;
            shell.request_redraw();
        }
        let choice = Choice::new(self, &state.shared);
        match key_pressed(event) {
            Some(KeyPress { key, press, .. }) if state.shared.is_open() => {
                choice.press(key, press, shell);
            }
            Some(KeyPress {
                key: Key::Named(Named::Enter),
                press,
                ..
            }) if state.focused && !press.is_some_and(|press| self.keymap.acts_at_once(press)) => {
                state.focused = false;
                choice.open(shell);
            }
            _ if pressed(event) && state.shared.is_open() => choice.click_beside(over, shell),
            _ if pressed(event) && over => choice.open(shell),
            _ if matches!(event, Event::Window(window::Event::RedrawRequested(_))) => {
                state.over = over;
            }
            _ if over != state.over => shell.request_redraw(),
            _ => {}
        }
    }

    /// Lets the keys that move between fields move to the list
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.custom(None, layout.bounds(), &mut Target(Act::Reach));
        operation.focusable(None, layout.bounds(), tree.state.downcast_mut::<State>());
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        pointer_over(cursor, layout.bounds())
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let State { shared, menu, .. } = tree.state.downcast_mut::<State>();
        if !shared.is_open() {
            return None;
        }
        let target = Rectangle::new(layout.position() + translation, layout.bounds().size());
        let menu = Menu::new(Choice::new(self, shared), menu, target);
        Some(overlay::Element::new(Box::new(menu)))
    }
}
