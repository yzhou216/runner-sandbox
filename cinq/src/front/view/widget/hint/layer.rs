// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Labels of hint mode, drawn in one layer over the window

use iced::{
    Element, Point, Rectangle, Size, Theme,
    widget::{container, pin, stack, text},
};
use itertools::Itertools;

use crate::front::{
    Message,
    view::{FONT, TEXT_SIZE, char_width, line_height, theme},
};

/// Room between the letters of a label and the edge of its box
pub(super) const PADDING: f32 = 2.0;

/// Returns the point of the window that the box of each label begins at, for
/// the labels of `labels`, each with its place and its number of letters.  The
/// box lies left of the place, so that it hides nothing of it, with its right
/// edge at the left edge of the place and its top at the top of the place,
/// except where that would leave the window.  There it lies right of the
/// place, with its left edge at the right edge of the place, so that what lies
/// at the left edge of the window stays in view, unless a box of another label
/// lies there, when it lies at the left edge of the window instead, or beside
/// the boxes that lie there, so that no label hides another.  The font is
/// monospace, so the width of a box needs no measuring of the text.
pub fn corners(labels: &[(Rectangle, usize)]) -> Vec<Point> {
    let lefts: Vec<_> = labels
        .iter()
        .map(|&(place, letters)| Point::new(place.x - width(letters), place.y))
        .collect();
    let on_left = lefts
        .iter()
        .zip(labels)
        .filter(|(left, _)| left.x >= 0.0)
        .map(|(&left, &(_, letters))| boxed(left, letters))
        .collect::<Vec<_>>();
    // Each label that does not lie on the left takes its place against every
    // box placed before it
    labels
        .iter()
        .zip(lefts)
        .scan(on_left, |taken, (&(place, letters), left)| {
            if left.x >= 0.0 {
                return Some(left);
            }
            let free = |at: &Point| {
                taken
                    .iter()
                    .all(|other| !other.intersects(&boxed(*at, letters)))
            };
            let right = Point::new(place.x + place.width, place.y);
            let edge = Point::new(0.0, place.y);
            let beside = taken
                .iter()
                .map(|other| Point::new(other.x + other.width, place.y))
                .sorted_by(|a, b| a.x.total_cmp(&b.x));
            let at = [right, edge]
                .into_iter()
                .chain(beside)
                .find(free)
                .unwrap_or(edge);
            taken.push(boxed(at, letters));
            Some(at)
        })
        .collect()
}

fn boxed(at: Point, letters: usize) -> Rectangle {
    Rectangle::new(at, Size::new(width(letters), line_height()))
}

pub fn width(letters: usize) -> f32 {
    letters as f32 * char_width() + 2.0 * PADDING
}

/// Returns a layer that draws each of `labels`, the letters still to type with
/// the point of the window that its box begins at.  The layer takes no event.
pub fn layer<'a>(labels: impl Iterator<Item = (&'a str, Point)>) -> Element<'a, Message> {
    stack(labels.map(|(letters, at)| pin(label(letters)).position(at).into())).into()
}

fn label(letters: &str) -> Element<'_, Message> {
    container(text(letters).font(FONT).size(TEXT_SIZE))
        .padding([0.0, PADDING])
        .style(|_: &Theme| container::Style {
            text_color: Some(theme::HINT_TEXT),
            background: Some(theme::HINT_BACKGROUND.into()),
            ..container::Style::default()
        })
        .into()
}
