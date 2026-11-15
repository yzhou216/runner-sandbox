// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rows of an open menu, one to an option, which the mouse highlights and
//! picks

use iced::{
    Element, Event, Length, Renderer, Theme,
    advanced::{
        Shell,
        layout::Layout,
        renderer::{self, Renderer as _},
    },
    border, mouse, overlay, padding,
    widget::{column, container, mouse_area, text},
};

use super::Choice;
use crate::front::{
    Message,
    view::widget::{
        dropdown::PADDING,
        pressed,
        wrap::{Intercept, wrap},
    },
};

pub(super) fn rows<'b>(choice: Choice<'b>) -> Element<'b, Message> {
    let highlight = choice.highlighted();
    let rows = choice.dropdown.names.iter().enumerate().map(|(row, name)| {
        container(text(name.as_str()).wrapping(text::Wrapping::None))
            .style(move |theme| {
                let style = container::Style::default();
                if highlight == Some(row) {
                    style.color(overlay::menu::default(theme).selected_text_color)
                } else {
                    style
                }
            })
            .padding(PADDING)
            .width(Length::Fill)
            .into()
    });
    wrap(
        mouse_area(column(rows)).interaction(mouse::Interaction::Pointer),
        Rows(choice),
    )
}

pub(super) struct Rows<'b>(pub(super) Choice<'b>);

impl Intercept for Rows<'_> {
    fn before(
        &self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let row = layout
            .children()
            .position(|row| cursor.is_over(row.bounds()));
        let highlight = &self.0.shared.highlight;
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. })
                if row.is_some() && row != self.0.highlighted() =>
            {
                highlight.set(row);
                shell.request_redraw();
            }
            _ if pressed(event) && row.is_some() => self.0.pick(row, shell),
            _ => {}
        }
        true
    }

    fn under(&self, layout: Layout<'_>, renderer: &mut Renderer, theme: &Theme) {
        let look = overlay::menu::default(theme);
        let row = self.0.highlighted();
        if let Some(row) = row.and_then(|row| layout.children().nth(row)) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: row.bounds().shrink(padding::horizontal(look.border.width)),
                    border: border::rounded(look.border.radius),
                    ..renderer::Quad::default()
                },
                look.selected_background,
            );
        }
    }
}
