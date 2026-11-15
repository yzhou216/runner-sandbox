// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Menu as an iced overlay

use iced::{
    Element, Event, Point, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Overlay, Shell,
        layout::{self, Layout},
        mouse, renderer,
        widget::{
            Id, Tree,
            operation::{self, Operation},
        },
    },
    overlay,
    widget::{container, operation::AbsoluteOffset, scrollable},
};

use super::{Choice, rows::rows};
use crate::front::{
    Message,
    keymap::KeyPress,
    reveal,
    view::{
        line_height,
        widget::{dropdown::PADDING, key_pressed, pressed},
    },
};

pub(in crate::front::view::widget::dropdown) struct Menu<'b> {
    choice: Choice<'b>,
    /// State of the widgets of the menu, kept in the state of the list
    tree: &'b mut Tree,
    content: Element<'b, Message>,
    /// Bounds of the drop-down list, which the menu opens next to
    target: Rectangle,
}

impl<'b> Menu<'b> {
    pub(in crate::front::view::widget::dropdown) fn new(
        choice: Choice<'b>,
        tree: &'b mut Tree,
        target: Rectangle,
    ) -> Self {
        let content = Element::from(container(scrollable(rows(choice))).style(|theme| {
            let look = overlay::menu::default(theme);
            container::Style {
                text_color: Some(look.text_color),
                background: Some(look.background),
                border: look.border,
                shadow: look.shadow,
                ..container::Style::default()
            }
        }));
        tree.diff(content.as_widget());
        Self {
            choice,
            tree,
            content,
            target,
        }
    }
}

impl Overlay<Message, Theme, Renderer> for Menu<'_> {
    /// A menu wider than its list, as the menu of a list that cuts its
    /// label short is, moves left as far as it has to for the window to
    /// hold it, as it does under a list at the right edge.
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let below = bounds.height - self.target.y - self.target.height;
        let above = self.target.y;
        let width = self
            .target
            .width
            .max(self.choice.dropdown.widest() + PADDING.x());
        let left = self.target.x.min(bounds.width - width).max(0.0);
        let limits =
            layout::Limits::new(Size::ZERO, Size::new(bounds.width - left, below.max(above)))
                .width(width);
        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let top = if below > above {
            self.target.y + self.target.height
        } else {
            self.target.y - node.size().height
        };
        node.move_to(Point::new(left, top))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        self.content
            .as_widget()
            .draw(self.tree, renderer, theme, style, layout, cursor, &bounds);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let shared = self.choice.shared;
        if !shared.is_open() {
            return;
        }
        match key_pressed(event) {
            Some(KeyPress { key, press, .. }) => self.choice.press(key, press, shell),
            None if pressed(event) && !cursor.is_over(layout.bounds()) => {
                self.choice.click_beside(cursor.is_over(self.target), shell);
            }
            None => {
                let bounds = layout.bounds();
                self.content.as_widget_mut().update(
                    self.tree, event, layout, cursor, renderer, clipboard, shell, &bounds,
                );
            }
        }
        if shared.reveal.replace(false)
            && let Some(row) = self.choice.highlighted()
        {
            let height = line_height() + PADDING.y();
            let top = height * row as f32;
            let reveal = &mut Reveal {
                top,
                bottom: top + height,
            };
            self.content
                .as_widget_mut()
                .operate(self.tree, layout, renderer, reveal);
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
}

/// Operation that scrolls the menu as little as it takes to show the span
/// from `top` to `bottom`, in pixels below the top of the first option
struct Reveal {
    top: f32,
    bottom: f32,
}

impl Operation for Reveal {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn operation::Scrollable,
    ) {
        if let Some(offset) = reveal(self.top, self.bottom, translation.y, bounds.height) {
            state.scroll_to(AbsoluteOffset {
                x: None,
                y: Some(offset),
            });
        }
    }
}
