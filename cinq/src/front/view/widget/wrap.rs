// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Widget that lays out and draws its content unchanged, around which an
//! [`Intercept`] handles events before and after the content sees them,
//! draws under the content, tells operations about it, and can take it out
//! of sight and reach while it stays in the widget tree

use iced::{
    Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Shell,
        layout::{self, Layout},
        mouse, overlay, renderer,
        widget::{
            Operation, Widget,
            tree::{self, Tree},
        },
    },
};

use super::hint::Hidden;
use crate::front::Message;

pub trait Intercept {
    /// Handles an event before the content sees it, and returns whether
    /// the content sees it at all
    fn before(
        &self,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _shell: &mut Shell<'_, Message>,
    ) -> bool {
        true
    }

    /// Returns the events that the content sees in place of the event, or
    /// nothing when it sees the event itself
    fn rewrite(
        &self,
        _content: &mut Element<'_, Message>,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<Vec<Event>> {
        None
    }

    fn after(
        &self,
        _content: &mut Element<'_, Message>,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _shell: &mut Shell<'_, Message>,
    ) {
    }

    fn under(&self, _layout: Layout<'_>, _renderer: &mut Renderer, _theme: &Theme) {}

    /// Tells an operation about the content, before the content tells it
    /// about itself
    fn operate(&self, _layout: Layout<'_>, _operation: &mut dyn Operation) {}

    /// Returns whether the content takes part: draws, sees events, and
    /// shows its overlays.  Content that does not take part still lays
    /// out, answers operations, and keeps the state that its widgets hold,
    /// but tells them when it begins and ends, so that those that look for
    /// places to act on leave the places in it out.
    fn shown(&self) -> bool {
        true
    }
}

struct Wrap<'a, I> {
    content: Element<'a, Message>,
    intercept: I,
}

pub fn wrap<'a>(
    content: impl Into<Element<'a, Message>>,
    intercept: impl Intercept + 'a,
) -> Element<'a, Message> {
    Element::new(Wrap {
        content: content.into(),
        intercept,
    })
}

impl<I: Intercept> Widget<Message, Theme, Renderer> for Wrap<'_, I> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
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
        if !self.intercept.shown() {
            return;
        }
        self.intercept.under(layout, renderer, theme);
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.intercept.operate(layout, operation);
        // Operations that find places to act on leave out those that the
        // operator cannot see
        let hidden = !self.intercept.shown();
        if hidden {
            operation.custom(None, layout.bounds(), &mut Hidden(true));
        }
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
        if hidden {
            operation.custom(None, layout.bounds(), &mut Hidden(false));
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
        if self.intercept.shown() && self.intercept.before(event, layout, cursor, shell) {
            let rewritten =
                self.intercept
                    .rewrite(&mut self.content, tree, event, layout, renderer);
            rewritten
                .as_deref()
                .unwrap_or(std::slice::from_ref(event))
                .iter()
                .for_each(|event| {
                    self.content.as_widget_mut().update(
                        tree, event, layout, cursor, renderer, clipboard, shell, viewport,
                    );
                });
            self.intercept
                .after(&mut self.content, tree, event, layout, renderer, shell);
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
        if !self.intercept.shown() {
            return mouse::Interaction::None;
        }
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.intercept
            .shown()
            .then(|| {
                self.content
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            })
            .flatten()
    }
}
