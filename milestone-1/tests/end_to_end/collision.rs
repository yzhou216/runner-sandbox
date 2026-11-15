//! Basic collision tests that don't use any other feature

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crate::common::*;
use bevy::input::keyboard::KeyCode;
use blox::{
    board::Block,
    config::*,
    data::{TetrominoType::*, *},
};

fn make_test_app<T: IntoIterator<Item = TetrominoType>>(tetrominos: T) -> Headless {
    Headless::new(
        GameConfig {
            bag: BagType::Deterministic,
            animate_title: true,
        },
        Some(Box::new(TestBag::new(tetrominos))),
    )
}

#[test]
fn basic_stacking() {
    // drop two blocks on top of each other with a slight adjustment
    let mut app = make_test_app([J, L, O, S]);
    app.set_relative_speed(24.0);
    let desired_duration = Duration::from_secs_f32(2.0 / 64.0);
    let mut sleep_debt = Duration::from_millis(0);
    let mut adjusted_sleep = |duration| {
        sleep_debt += duration;
        if desired_duration < sleep_debt {
            sleep_debt -= desired_duration;
        } else {
            sleep(desired_duration - sleep_debt);
            sleep_debt = Duration::from_millis(0);
        }
    };
    // lock the J
    for _ in 0..23 {
        let before = Instant::now();
        app.update();
        let after = Instant::now();
        adjusted_sleep(after.duration_since(before));
    }
    // move and lock the L
    for i in 0..20 {
        let before = Instant::now();
        if i == 0 {
            // this includes two updates already
            app.release_then_press(KeyCode::ArrowLeft);
        } else {
            app.update();
        }
        let after = Instant::now();
        adjusted_sleep(after.duration_since(before));
    }
    let mut expected_obstacles = vec![];
    expected_obstacles.extend(
        [Cell(3, 0), Cell(4, 0), Cell(5, 0), Cell(5, 1)].map(|cell| Block {
            cell,
            color: get_tetromino(J).color,
        }),
    );
    expected_obstacles.extend(
        [Cell(2, 1), Cell(3, 1), Cell(4, 1), Cell(2, 2)].map(|cell| Block {
            cell,
            color: get_tetromino(L).color,
        }),
    );
    assert_eq!(&app.obstacles(), &expected_obstacles,);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(4, 18), Cell(4, 19), Cell(5, 18), Cell(5, 19)],
            center: (4.5, 18.5),
            color: get_tetromino(O).color
        }]
    );
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(1, 2), Cell(2, 2), Cell(2, 3), Cell(3, 3)],
            center: (2.0, 2.0),
            color: get_tetromino(S).color
        }]
    );
}

// Update the app and sleep for given duration in between (on average) for the
// given number of repetitions.
fn sleep_with_debt(app: &mut Headless, desired_duration: Duration, repetitions: usize) {
    let mut sleep_debt = Duration::ZERO;
    let mut adjusted_sleep = |duration| {
        sleep_debt += duration;
        if desired_duration < sleep_debt {
            sleep_debt -= desired_duration;
        } else {
            sleep(desired_duration - sleep_debt);
            sleep_debt = Duration::ZERO;
        }
    };

    for _ in 0..repetitions {
        let before = Instant::now();
        app.update();
        let after = Instant::now();
        adjusted_sleep(after - before);
    }
}

#[test]
fn basic_game_over() {
    // drop blocks until game over, using the deterministic bag.
    let mut app = make_test_app([]);
    app.app_mut()
        .world_mut()
        .commands()
        .add_observer(observe_game_over);
    for piece_idx in 0..10 {
        for drop_idx in 0..(20 - piece_idx * 2) {
            app.release_then_press(KeyCode::ArrowDown);
            let world = app.app_mut().world_mut();
            assert!(
                world.get_resource::<GameIsOver>().is_none(),
                "Game ended prematurely on drop #{drop_idx} of the piece #{piece_idx}"
            );
        }
        // make sure that the piece is lodged
        sleep_with_debt(&mut app, Duration::from_secs_f32(1.0 / 64.0), 64);
    }
    sleep_with_debt(&mut app, Duration::from_secs_f32(1.0 / 64.0), 48);

    let world = app.app_mut().world_mut();
    assert!(
        world.get_resource::<GameIsOver>().is_some(),
        "Game should have ended."
    );
}
