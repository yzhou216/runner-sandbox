// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drop-down list as an iced widget

use iced::{
    Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Renderer as _, Shell,
        layout::{self, Layout},
        mouse, overlay, renderer,
        text::{self, Renderer as _, Text},
        widget::{
            Operation, Widget,
            tree::{self, Tree},
        },
    },
    alignment,
    keyboard::{Key, key::Named},
    widget::{
        pick_list::{self, Status},
        text_input,
    },
    window,
};

use super::{
    Dropdown, PADDING,
    menu::{Choice, Menu},
    shorten::shortened,
    state::State,
};
use crate::front::{
    Message,
    keymap::KeyPress,
    view::{
        FONT, TEXT_SIZE, char_width, line_height,
        widget::{key_pressed, pointer_over, pressed},
    },
};

/// Room for the arrow, which is about as wide as the text is high
const ARROW: f32 = TEXT_SIZE.0 + PADDING.left;

/// Returns a label on one line in the default font and size.  It goes by
/// its top left corner, where iced's own pick list goes by its middle,
/// since iced's software renderer redraws a changed text only below and to
/// the right of the point it is drawn at.
fn label<C>(content: C, width: f32) -> Text<C> {
    Text {
        content,
        bounds: Size::new(width, line_height()),
        size: TEXT_SIZE,
        line_height: text::LineHeight::default(),
        font: FONT,
        align_x: text::Alignment::Default,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::default(),
        wrapping: text::Wrapping::None,
    }
}

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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let is_hovered = cursor.is_over(bounds);
        let status = match (state.shared.is_open(), is_hovered) {
            (true, _) => Status::Opened { is_hovered },
            (false, true) => Status::Hovered,
            (false, false) => Status::Active,
        };
        let look = pick_list::default(theme, status);
        // Border of a focused list is that of a focused text field
        let border = if state.focused {
            let focused = text_input::Status::Focused { is_hovered };
            look.border
                .color(text_input::default(theme, focused).border.color)
        } else {
            look.border
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border,
                ..renderer::Quad::default()
            },
            look.background,
        );
        // Arrow drawn by its right end as iced's pick list draws it, since
        // it never changes apart from the box, which redraws around it
        let arrow = Text {
            font: Renderer::ICON_FONT,
            align_x: text::Alignment::Right,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Basic,
            ..label(Renderer::ARROW_DOWN_ICON.to_string(), bounds.width)
        };
        let end = Point::new(bounds.x + bounds.width - PADDING.right, bounds.center_y());
        renderer.fill_text(arrow, end, look.handle_color, *viewport);
        let (name, color) = match self.chosen {
            Some(row) => (self.names[row].clone(), look.text_color),
            None => (
                self.placeholder.clone().unwrap_or_default(),
                look.placeholder_color,
            ),
        };
        let room = bounds.width - PADDING.x() - ARROW;
        let name = label(shortened(&name, (room / char_width()) as usize), room);
        let start = Point::new(
            bounds.x + PADDING.left,
            bounds.center_y() - line_height() / 2.0,
        );
        renderer.fill_text(name, start, color, *viewport);
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
