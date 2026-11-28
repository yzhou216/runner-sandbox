// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the events that the subscription turns into messages

use std::{
    thread,
    time::{Duration, Instant},
};

use iced::{Event, event::Status, mouse, window};

use super::{clicked, in_window, side_pressed};
use crate::{
    back::{Audio, Fault, Lights, Network, Video},
    front::{App, Cinq, Message, update::Keeper},
};

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

#[test]
fn subscription_asks_for_a_look_while_an_audio_device_has_failed() {
    let (lights, light_reports) = Lights::open();
    let mut cinq = Cinq {
        app: App::default(),
        // No audio host is named cinq, so this device is never there
        audio: Audio::open(Some("cinq:none")),
        video: Video::default(),
        lights,
        light_reports,
        network: Network::open(),
        keeper: Keeper::new(None),
    };
    let idle = cinq.subscription().units();
    let deadline = Instant::now() + Duration::from_secs(4);
    while !matches!(
        cinq.app.problems.audio.first(),
        Some((_, Fault::Missing(_)))
    ) {
        assert!(Instant::now() < deadline, "the device has not answered");
        let _ = cinq.update(Message::NetworkChecked);
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        cinq.subscription().units(),
        idle + 1,
        "a timer, while the device is tried again"
    );
}
