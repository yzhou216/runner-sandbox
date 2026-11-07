// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures that fill a region one over another, each in the middle and as
//! large as the region holds it whole
//!
//! iced's renderer on the GPU puts every large new image into a texture of
//! its own, which software renderers and slow GPUs pay dearly for, and
//! draws nothing in its place until a thread of its own has uploaded it.
//! The picture of a video cue changes nearly every frame, so these pictures
//! draw through a shader of their own instead, which keeps one texture for
//! each cue and fills it again with each new picture of the cue before the
//! frame that shows it.  The software renderer draws no shader, and draws
//! the pictures as images, each in the frame that shows it.

mod pipeline;
#[cfg(test)]
mod tests;

use iced::{
    ContentFit, Element, Length, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Widget,
        image::{self as drawing, Renderer as _},
        layout::{self, Layout},
        mouse, renderer,
        widget::tree::{self, Tree},
    },
    widget::{image, shader},
};

use crate::{back::CueId, front::Message};

#[derive(Debug, Clone)]
pub struct Layer {
    pub cue: CueId,
    pub picture: image::Handle,
    pub opacity: f32,
}

/// Returns pictures that fill a region, with `layers` from the bottom up
pub fn pictures<'a>(layers: Vec<Layer>) -> Element<'a, Message> {
    Element::new(Pictures(layers))
}

struct Pictures(Vec<Layer>);

impl Widget<Message, Theme, Renderer> for Pictures {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    // Unit state for the shader to read as it draws, under a tag of its own,
    // since iced would keep the missing state of a stateless widget that was
    // here before, whose tag is that of a unit
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Self>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(())
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fill)
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
        if matches!(renderer, Renderer::Primary(_)) {
            shader::<Message, _>(self).draw(tree, renderer, theme, style, layout, cursor, viewport);
        } else {
            let bounds = layout.bounds();
            self.0.iter().for_each(|layer| {
                if let Some((size, _)) = rgba(&layer.picture) {
                    let picture = drawing::Image {
                        opacity: layer.opacity,
                        snap: true,
                        ..drawing::Image::new(layer.picture.clone())
                    };
                    renderer.draw_image(picture, fitted(bounds, size), bounds);
                }
            });
        }
    }
}

impl shader::Program<Message> for Pictures {
    type State = ();
    type Primitive = pipeline::Pictures;

    fn draw(&self, _state: &(), _cursor: mouse::Cursor, _bounds: Rectangle) -> Self::Primitive {
        pipeline::Pictures(self.0.clone())
    }
}

/// Returns the size and the pixels of a picture, which the stage makes from
/// the decoded pixels that fill it, or nothing for an empty picture or one
/// of any other kind, which draws nothing
fn rgba(picture: &image::Handle) -> Option<(Size<u32>, &[u8])> {
    let image::Handle::Rgba {
        width,
        height,
        pixels,
        ..
    } = picture
    else {
        return None;
    };
    let filled = pixels.len() as u64 == 4 * u64::from(*width) * u64::from(*height);
    (filled && !pixels.is_empty()).then(|| (Size::new(*width, *height), pixels.as_ref()))
}

fn fitted(bounds: Rectangle, size: Size<u32>) -> Rectangle {
    let size = Size::new(size.width as f32, size.height as f32);
    let fitted = ContentFit::Contain.fit(size, bounds.size());
    Rectangle::new(
        bounds.center() - Vector::new(fitted.width, fitted.height) * 0.5,
        fitted,
    )
}
