// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the events that the subscription turns into messages

use iced::{Event, event::Status, mouse, window};

use super::side_pressed;
use crate::front::Message;

#[test]
fn only_the_back_and_forward_buttons_of_the_mouse_go_along() {
    let window = window::Id::unique();
    let pressed = |button| {
        side_pressed(
            Event::Mouse(mouse::Event::ButtonPressed(button)),
            Status::Ignored,
            window,
        )
    };
    assert!(matches!(
        pressed(mouse::Button::Back),
        Some(Message::SidePressed(id, mouse::Button::Back)) if id == window
    ));
    assert!(matches!(
        pressed(mouse::Button::Forward),
        Some(Message::SidePressed(id, mouse::Button::Forward)) if id == window
    ));
    assert!(pressed(mouse::Button::Left).is_none());
    let released = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Back));
    assert!(side_pressed(released, Status::Ignored, window).is_none());
}
