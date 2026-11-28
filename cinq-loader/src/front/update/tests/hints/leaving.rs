// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what leaves hint mode: keys that are no letters, clicks, the wheel, resizes, and changes of what lies below

use iced::window;

use super::*;
use crate::front::Problems;

#[test]
fn a_key_that_is_no_letter_leaves_the_mode_and_does_nothing_else() {
    ["ArrowDown", "Control+c", "Tab", "Enter", "Control+d"]
        .into_iter()
        .for_each(|keys| {
            let after = typing(labelled(), keys);
            assert!(after.hints.is_none(), "{keys}");
            assert_eq!(after.selected, Some(CueId(2)), "{keys}");
            assert!(!after.outputs_shown, "{keys}");
        });
}

#[test]
fn go_and_panic_act_as_everywhere_and_leave_the_mode() {
    let panicked = typing(helping(labelled()), "Escape");
    assert!(panicked.hints.is_none());
    assert!(!panicked.help, "Panic closes the help page");
    assert!(typing(labelled(), "Space").hints.is_none());
}

#[test]
fn a_held_letter_does_not_choose_the_labels_that_it_begins() {
    let (held, task) = labelled().typed(press(Key::Character("a".into())), true);
    assert!(held.hints.is_some());
    assert!(!held.outputs_shown);
    assert_eq!(task.units(), 0);
}

#[test]
fn a_click_or_a_turn_of_the_wheel_leaves_the_mode() {
    [Message::Clicked, Message::Remeasure]
        .into_iter()
        .for_each(|message| {
            let (left, _) = labelled().next(message.clone());
            assert!(left.hints.is_none(), "{message:?}");
        });
}

#[test]
fn a_resize_of_the_main_window_leaves_the_mode_and_that_of_another_does_not() {
    let size = Size::new(800.0, 600.0);
    let (other, _) = labelled().next(Message::Resized(window::Id::unique(), size));
    assert!(other.hints.is_some());
    let (windowed, main) = windowed(labelled());
    let (resized, _) = windowed.next(Message::Resized(main, size));
    assert!(resized.hints.is_none());
}

#[test]
fn a_change_of_the_active_list_leaves_the_mode_and_the_same_list_keeps_it() {
    let before = labelled().above_places();
    assert!(labelled().hints_after(Some(&before)).hints.is_some());
    let running = after(with_sound(labelled()), [Message::Go]);
    assert!(running.hints_after(Some(&before)).hints.is_none());
}

#[test]
fn a_warning_that_comes_or_goes_leaves_the_mode_and_a_mode_that_began_now_stays() {
    let warned = || App {
        problems: Problems::default().with_osc(Some("osc_listen: cannot listen".to_owned())),
        ..labelled()
    };
    let before = labelled().above_places();
    assert!(warned().hints_after(Some(&before)).hints.is_none());
    let cleared = warned().above_places();
    assert!(labelled().hints_after(Some(&cleared)).hints.is_none());
    assert!(warned().hints_after(None).hints.is_some());
}
