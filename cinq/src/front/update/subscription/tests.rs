// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the events that the subscription turns into messages

use iced::{Event, event::Status, mouse, window};

use super::{clicked, in_window, side_pressed};
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

#[test]
fn clicks_and_wheel_turns_in_the_stage_window_are_left_out() {
    let (main, stage) = (window::Id::unique(), window::Id::unique());
    let heard = |event, window| {
        clicked(event, Status::Ignored, window)
            .and_then(|(window, message)| in_window(Some(main), window, message))
    };
    let pressed = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let wheeled = Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
    });
    assert!(matches!(
        heard(pressed.clone(), main),
        Some(Message::Clicked)
    ));
    assert!(matches!(
        heard(wheeled.clone(), main),
        Some(Message::Remeasure)
    ));
    assert!(heard(pressed, stage).is_none());
    assert!(heard(wheeled, stage).is_none());
}
