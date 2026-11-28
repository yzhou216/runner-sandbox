// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys typed in hint mode: the letters that choose a place,
//! those that take a letter back or leave the mode, and the keys and clicks
//! that act as everywhere

mod choosing;
mod leaving;
mod toggling;

use iced::{Point, Rectangle, Size};

use super::*;
use crate::front::view::{Act, Site, hint};

fn place(act: Act, x: f32) -> Site {
    let bounds = Rectangle::new(Point::new(x, 10.0), Size::new(20.0, 20.0));
    Site {
        shown: bounds,
        at: bounds,
        act,
    }
}

/// Returns the state with hint mode on, as the widgets told that these places
/// lie in the window, a list switch, the outputs, and a field, which the
/// labels `s`, `a`, and `d` stand for
fn labelled() -> App {
    let places = vec![
        place(Act::Send(Message::SwitchList(1)), 0.0),
        place(Act::Send(Message::ShowOutputs), 60.0),
        place(Act::Reach, 120.0),
    ];
    let (app, _) = show(Some(CueId(2))).hinting();
    app.hinted(places)
}

/// Returns the state with hint mode on over more places than letters, so that
/// every label has two letters, the first of them leading to the places
/// `Message::Row(n)` in the order of the letters
fn many_labelled() -> App {
    let places = (1..=20).map(|row| place(Act::Send(Message::Row(row)), 0.0));
    let (app, _) = show(Some(CueId(2))).hinting();
    app.hinted(places.collect())
}

/// Returns the letters of the labels that the layer shows
fn shown(app: &App) -> Vec<&str> {
    app.hints
        .as_ref()
        .map(|hints| hints.labels().map(|(letters, _)| letters).collect())
        .unwrap_or_default()
}
