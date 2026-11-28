// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Widget that Tab reaches and Enter acts on, for iced's checkbox and button
//!
//! It draws the widget of iced inside the border of a focused text field while
//! it has the keyboard, and takes part in the operations that move the
//! keyboard from field to field.  Enter is handled by [`keyed`].  Space stays
//! GO and Escape stays Panic.

use iced::{
    Element, Event, Length, Rectangle, Renderer, Size, Theme,
    advanced::{
        Clipboard, Shell,
        layout::{self, Layout},
        mouse, renderer,
        widget::{
            Id, Operation, Widget,
            tree::{self, Tree},
        },
    },
};

use super::{State, keyed, mark};
use crate::front::{Message, keymap::Keymap, view};

/// Room that the label of a checkbox keeps after its last letter, within the
/// border
const ROOM: f32 = 4.0;

pub struct Focused<'a> {
    /// iced's widget, which draws this one
    look: Element<'a, Message>,
    /// Message that Enter sends, which flips the box or presses the button,
    /// unless the widget cannot change
    act: Option<Message>,
    /// Whether Enter gives the keyboard back to the cue list after it acts
    leaves: bool,
    keymap: &'a Keymap,
    /// Identity that the operations find the widget by
    id: &'static str,
    /// Room that the widget keeps after its look, within the border
    room: f32,
}

impl<'a> Focused<'a> {
    /// Returns a checkbox that Enter flips by sending `flip`, or that takes no
    /// keyboard without it
    pub fn check(
        look: Element<'a, Message>,
        flip: Option<Message>,
        leaves: bool,
        keymap: &'a Keymap,
    ) -> Self {
        Self {
            look,
            act: flip,
            leaves,
            keymap,
            id: view::CHECK,
            room: ROOM,
        }
    }

    /// Returns a button that Enter presses by sending `press`
    pub fn button(
        look: Element<'a, Message>,
        press: Message,
        leaves: bool,
        keymap: &'a Keymap,
    ) -> Self {
        Self {
            act: Some(press),
            id: view::BUTTON,
            room: 0.0,
            ..Self::check(look, None, leaves, keymap)
        }
    }

    /// Returns the layout of iced's widget, which lies in the room of this one
    fn look<'b>(layout: Layout<'b>) -> Layout<'b> {
        layout
            .children()
            .next()
            .expect("the layout of a focused widget holds that of iced's")
    }
}

impl Widget<Message, Theme, Renderer> for Focused<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State { focused: false })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.look)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.look));
    }

    fn size(&self) -> Size<Length> {
        self.look.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let room = Size::new(self.room, 0.0);
        let look =
            self.look
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits.shrink(room));
        layout::Node::with_children(look.size().expand(room), vec![look])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.look.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            Self::look(layout),
            cursor,
            viewport,
        );
        if tree.state.downcast_ref::<State>().focused {
            mark(renderer, theme, layout.bounds());
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let taken = keyed(
            state,
            event,
            self.keymap,
            self.act.as_ref(),
            self.leaves,
            shell,
        );
        if !taken {
            self.look.as_widget_mut().update(
                &mut tree.children[0],
                event,
                Self::look(layout),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    /// Lets the keys that move between fields move to the widget, unless it
    /// cannot change
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.act.is_some() {
            operation.focusable(
                Some(&Id::new(self.id)),
                layout.bounds(),
                tree.state.downcast_mut::<State>(),
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.look.as_widget().mouse_interaction(
            &tree.children[0],
            Self::look(layout),
            cursor,
            viewport,
            renderer,
        )
    }
}
