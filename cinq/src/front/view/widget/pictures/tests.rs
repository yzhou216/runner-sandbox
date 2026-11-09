// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where a picture fits in its region, of which pictures draw,
//! and of the state that the shader reads

use iced::{
    Element, Point, Rectangle, Size,
    advanced::widget::{Tree, tree},
    widget::{image, space},
};

use super::{fitted, pictures, rgba};
use crate::front::Message;

#[test]
fn wide_picture_spans_the_width_in_the_middle_of_the_height() {
    let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(960.0, 720.0));
    let expected = Rectangle::new(Point::new(10.0, 110.0), Size::new(960.0, 540.0));
    assert_eq!(fitted(bounds, Size::new(1920, 1080)), expected);
}

#[test]
fn tall_picture_spans_the_height_in_the_middle_of_the_width() {
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(1000.0, 500.0));
    let expected = Rectangle::new(Point::new(375.0, 0.0), Size::new(250.0, 500.0));
    assert_eq!(fitted(bounds, Size::new(500, 1000)), expected);
}

#[test]
fn picture_gives_its_size_and_pixels() {
    let pixels = vec![7; 4 * 3 * 2];
    let picture = image::Handle::from_rgba(3, 2, pixels.clone());
    assert_eq!(rgba(&picture), Some((Size::new(3, 2), pixels.as_slice())));
}

#[test]
fn picture_whose_pixels_fall_short_or_that_is_empty_draws_nothing() {
    let short = image::Handle::from_rgba(3, 2, vec![7; 4 * 3]);
    let empty = image::Handle::from_rgba(0, 2, Vec::new());
    assert_eq!(rgba(&short), None);
    assert_eq!(rgba(&empty), None);
}

#[test]
fn pictures_drawn_where_a_stateless_widget_was_have_the_state_that_the_shader_reads() {
    let space: Element<'_, Message> = space().into();
    let mut tree = Tree::new(space.as_widget());
    tree.diff(pictures(Vec::new()).as_widget());
    assert!(matches!(tree.state, tree::State::Some(_)));
}
