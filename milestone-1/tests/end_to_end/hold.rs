//! Hold window tests

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crate::common::*;
use bevy::input::keyboard::KeyCode;
use blox::{
    config::*,
    data::{TetrominoType::*, *},
    hold::*,
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

fn wait_1_frame(app: &mut Headless, time_spent: Duration) {
    let desired_time = Duration::from_secs_f32(1.0 / 64.0) / app.relative_speed().round() as u32;
    if desired_time > time_spent {
        sleep(desired_time - time_spent);
    }
}

#[test]
fn first_hold() {
    let mut app = make_test_app([I, T, S]);
    app.update();
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 19), Cell(4, 19), Cell(5, 19), Cell(6, 19)],
            center: (4.5, 18.5),
            color: get_tetromino(I).color
        }]
    );
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(2, 3), Cell(1, 2), Cell(2, 2), Cell(3, 2)],
            center: (2.0, 2.0),
            color: get_tetromino(T).color
        }]
    );
    assert_eq!(app.tetrominos::<Hold>(), vec![]);

    // trigger a hold
    app.release_then_press(KeyCode::KeyX);
    // wait for 1 frame to update the next window
    wait_1_frame(&mut app, Duration::ZERO);
    app.update();

    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(4, 19), Cell(3, 18), Cell(4, 18), Cell(5, 18)],
            center: (4.0, 18.0),
            color: get_tetromino(T).color
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
    assert_eq!(
        app.tetrominos::<Hold>(),
        vec![Tetromino {
            cells: [Cell(1, 3), Cell(2, 3), Cell(3, 3), Cell(4, 3)],
            center: (2.5, 2.5),
            color: get_tetromino(I).color
        }]
    );
}

#[test]
fn next_hold() {
    let mut app = make_test_app([I, T, S, L]);
    app.update();
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 19), Cell(4, 19), Cell(5, 19), Cell(6, 19)],
            center: (4.5, 18.5),
            color: get_tetromino(I).color
        }]
    );
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(2, 3), Cell(1, 2), Cell(2, 2), Cell(3, 2)],
            center: (2.0, 2.0),
            color: get_tetromino(T).color
        }]
    );
    assert_eq!(app.tetrominos::<Hold>(), vec![]);

    // trigger a hold
    app.release_then_press(KeyCode::KeyX);
    // wait for 1 frame to update the next window
    wait_1_frame(&mut app, Duration::ZERO);
    app.update();

    // wait until this piece locks in
    app.set_relative_speed(16.0);
    let desired_duration = Duration::from_secs_f32(3.0 / 64.0);
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
    for _ in 0..24 {
        let before = Instant::now();
        app.update();
        let after = Instant::now();
        adjusted_sleep(after.duration_since(before));
    }
    app.set_relative_speed(1.0);

    // trigger a hold
    app.release_then_press(KeyCode::KeyX);
    wait_1_frame(&mut app, Duration::ZERO);
    app.update();

    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 18), Cell(4, 18), Cell(5, 18), Cell(6, 18)],
            center: (4.5, 17.5),
            color: get_tetromino(I).color
        }]
    );
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(1, 2), Cell(2, 2), Cell(3, 2), Cell(1, 3)],
            center: (2.0, 2.0),
            color: get_tetromino(L).color
        }]
    );
    assert_eq!(
        app.tetrominos::<Hold>(),
        vec![Tetromino {
            cells: [Cell(1, 2), Cell(2, 2), Cell(2, 3), Cell(3, 3)],
            center: (2.0, 2.0),
            color: get_tetromino(S).color
        }]
    );
}
