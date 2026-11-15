// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stage window, which shows the pictures of the picture and video cues
//! that run
//!
//! The cue that fired last shows on top.  As in QLab, those that fired
//! before it show around a picture of another shape than the window, and
//! through one that fades, so that a picture fading out as its cue stops
//! fades to whatever shows once it has gone.  The pointer stays hidden over
//! the stage, out of the audience's sight.

use iced::{Element, mouse, widget::mouse_area};

use super::widget::{Layer, pictures};
use crate::front::{App, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let layers = app
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
    mouse_area(pictures(layers))
        .interaction(mouse::Interaction::Hidden)
        .into()
}
