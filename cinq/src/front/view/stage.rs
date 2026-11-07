// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stage window, which shows the pictures of the picture and video cues
//! that run
//!
//! Each picture fits the window whole, on the black background of the
//! theme, which shows around a picture whose shape differs from that of the
//! window, and on its own while no picture shows.  The cue that fired last
//! shows on top, and hides those that fired before it while its picture is
//! wholly opaque, so that a picture fading out as its cue stops fades to
//! whatever shows once it has gone.  Each picture draws in the first frame
//! that shows it, through [`pictures`].  The pointer stays hidden over the
//! stage, out of the audience's sight.

use iced::{Element, mouse, widget::mouse_area};

use super::widget::{Layer, pictures};
use crate::front::{App, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let layers: Vec<Layer> = app
        .workspace
        .active()
        .iter()
        .filter(|active| active.shows_picture())
        .filter_map(|active| {
            Some(Layer {
                cue: active.id,
                picture: app.stage.picture(active.id)?.clone(),
                opacity: active.opacity(),
            })
        })
        .collect();
    let hidden = layers
        .iter()
        .rposition(|layer| layer.opacity >= 1.0)
        .unwrap_or(0);
    let shown = layers.into_iter().skip(hidden).collect();
    mouse_area(pictures(shown))
        .interaction(mouse::Interaction::Hidden)
        .into()
}
