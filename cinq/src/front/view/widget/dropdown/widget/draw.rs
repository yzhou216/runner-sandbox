// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Look of a drop-down list when it is closed

use iced::{
    Border, Color, Point, Rectangle, Renderer, Size, Theme,
    advanced::{
        Renderer as _,
        layout::Layout,
        mouse, renderer,
        text::{self, Renderer as _, Text},
        widget::tree::Tree,
    },
    alignment,
    widget::{
        pick_list::{self, Status},
        text_input,
    },
};

use super::{
    super::{Dropdown, PADDING, shorten::shortened, state::State},
    ARROW,
};
use crate::front::view::{FONT, TEXT_SIZE, char_width, line_height, theme};

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

impl Dropdown<'_> {
    pub(super) fn drawn(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
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
        let look = self.look(theme, status);
        let border = focused_border(&look, theme, state.focused, is_hovered);
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border,
                ..renderer::Quad::default()
            },
            look.background,
        );
        let (name, start, color) = self.name(&look, bounds);
        let (arrow, end) = arrow(bounds);
        // The software renderer clips cached text to its layer alone, and a
        // viewport as large as the layer never narrows that, so that a list
        // across the edge of a scrollable would draw its text outside it.
        // A layer of its own as large as the shown part of the list clips it.
        if let Some(shown) = viewport.intersection(&bounds) {
            renderer.with_layer(shown, |renderer| {
                renderer.fill_text(arrow, end, look.handle_color, *viewport);
                renderer.fill_text(name, start, color, *viewport);
            });
        }
    }

    fn look(&self, theme: &Theme, status: Status) -> pick_list::Style {
        let look = pick_list::default(theme, status);
        // List that an option of another menu depends on wears the colors of
        // the highlight of a menu, as that option does
        if self.lit {
            let menu = iced::overlay::menu::default(theme);
            pick_list::Style {
                background: menu.selected_background,
                text_color: menu.selected_text_color,
                placeholder_color: menu.selected_text_color,
                handle_color: menu.selected_text_color,
                ..look
            }
        } else {
            look
        }
    }

    /// Chosen name, or the placeholder, shortened to its room, with where
    /// it starts and its color
    fn name(&self, look: &pick_list::Style, bounds: Rectangle) -> (Text<String>, Point, Color) {
        let (name, color) = match self.chosen {
            Some(row) => (self.names[row].as_str(), look.text_color),
            None => (
                self.placeholder.as_deref().unwrap_or_default(),
                self.placeholder_color.unwrap_or(look.placeholder_color),
            ),
        };
        let room = bounds.width - PADDING.x() - ARROW;
        let name = label(shortened(name, (room / char_width()) as usize), room);
        let start = Point::new(
            bounds.x + PADDING.left,
            bounds.center_y() - line_height() / 2.0,
        );
        (name, start, color)
    }
}

/// Arrow with where it ends
fn arrow(bounds: Rectangle) -> (Text<String>, Point) {
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
    (arrow, end)
}

/// Border of a focused list is that of a focused text field
fn focused_border(
    look: &pick_list::Style,
    theme: &Theme,
    focused: bool,
    is_hovered: bool,
) -> Border {
    if focused {
        let focused = text_input::Status::Focused { is_hovered };
        theme::focused(
            look.border
                .color(text_input::default(theme, focused).border.color),
            focused,
        )
    } else {
        look.border
    }
}
