//! Baseline tests

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

// Test four rotations of the given tetromino
fn test_rotation(app: &mut Headless, key: KeyCode, tetrominos: [Tetromino; 4]) {
    app.pressed_keys(&[key]);
    app.update();
    app.pressed_keys(&[KeyCode::ArrowUp]);
    app.update();
    assert_eq!(app.tetrominos::<Active>(), vec![tetrominos[0]]);
    app.release_then_press(KeyCode::ArrowUp);
    assert_eq!(app.tetrominos::<Active>(), vec![tetrominos[1]]);
    app.release_then_press(KeyCode::ArrowUp);
    assert_eq!(app.tetrominos::<Active>(), vec![tetrominos[2]]);
    app.release_then_press(KeyCode::ArrowUp);
    assert_eq!(app.tetrominos::<Active>(), vec![tetrominos[3]]);
    app.release_then_press(KeyCode::ArrowUp);
    assert_eq!(app.tetrominos::<Active>(), vec![tetrominos[0]]);
}

#[test]
fn i_spawn() {
    let mut app = make_test_app([I]);
    app.update();
    assert!(app.obstacles().is_empty());
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 19), Cell(4, 19), Cell(5, 19), Cell(6, 19)],
            center: (4.5, 18.5),
            color: get_tetromino(I).color,
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

#[test]
fn j_spawn() {
    let mut app = make_test_app([J]);
    app.update();
    assert!(app.obstacles().is_empty());
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 18), Cell(4, 18), Cell(5, 18), Cell(5, 19)],
            center: (4.0, 18.0),
            color: get_tetromino(J).color,
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

#[test]
fn gravity1() {
    // drop 1 block 19 times then lock it
    let mut app = make_test_app([J, O]);
    app.set_relative_speed(24.0);
    let desired_duration = Duration::from_secs_f32(2.0 / 64.0);
    let mut sleep_debt = Duration::from_millis(0);
    for _ in 0..23 {
        let before = Instant::now();
        app.update();
        let after = Instant::now();
        sleep_debt += after.duration_since(before);
        if desired_duration < sleep_debt {
            sleep_debt -= desired_duration;
        } else {
            sleep(desired_duration - sleep_debt);
            sleep_debt = Duration::from_millis(0);
        }
    }
    assert_eq!(
        &app.obstacles(),
        &[Cell(3, 0), Cell(4, 0), Cell(5, 0), Cell(5, 1),].map(|cell| Block {
            cell,
            color: get_tetromino(J).color
        })
    );
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

#[test]
fn shift1() {
    // shift a tetromino left, left, down
    let mut app = make_test_app([S, O]);
    app.release_then_press(KeyCode::ArrowLeft);
    app.release_then_press(KeyCode::ArrowLeft);
    app.release_then_press(KeyCode::ArrowDown);
    assert_eq!(&app.obstacles(), &[]);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(1, 17), Cell(2, 17), Cell(2, 18), Cell(3, 18)],
            center: (2.0, 17.0),
            color: get_tetromino(S).color
        }]
    );
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(2, 2), Cell(2, 3), Cell(3, 2), Cell(3, 3)],
            center: (2.5, 2.5),
            color: get_tetromino(O).color
        }]
    );
}

#[test]
fn shift2() {
    // shift a tetromino left, right, left, right
    let mut app = make_test_app([Z, O]);
    app.update();
    app.release_then_press(KeyCode::ArrowLeft);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(2, 19), Cell(3, 18), Cell(3, 19), Cell(4, 18)],
            center: (3.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    app.release_then_press(KeyCode::ArrowRight);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 19), Cell(4, 18), Cell(4, 19), Cell(5, 18)],
            center: (4.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    app.release_then_press(KeyCode::ArrowLeft);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(2, 19), Cell(3, 18), Cell(3, 19), Cell(4, 18)],
            center: (3.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    app.release_then_press(KeyCode::ArrowRight);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 19), Cell(4, 18), Cell(4, 19), Cell(5, 18)],
            center: (4.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    assert_eq!(&app.obstacles(), &[]);
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(2, 2), Cell(2, 3), Cell(3, 2), Cell(3, 3)],
            center: (2.5, 2.5),
            color: get_tetromino(O).color
        }]
    );
}

#[test]
fn shift3() {
    // shift all the way to the left and more until collision
    let mut app = make_test_app([Z, O]);
    app.update();
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 19), Cell(4, 18), Cell(4, 19), Cell(5, 18)],
            center: (4.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    for _ in 0..6 {
        app.release_then_press(KeyCode::ArrowLeft);
    }
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(0, 19), Cell(1, 18), Cell(1, 19), Cell(2, 18)],
            center: (1.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    assert_eq!(&app.obstacles(), &[]);
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(2, 2), Cell(2, 3), Cell(3, 2), Cell(3, 3)],
            center: (2.5, 2.5),
            color: get_tetromino(O).color
        }]
    );
}

#[test]
fn shift4() {
    // shift all the way to the right and more until collision
    let mut app = make_test_app([Z, I]);
    app.update();
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 19), Cell(4, 18), Cell(4, 19), Cell(5, 18)],
            center: (4.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    for _ in 0..6 {
        app.release_then_press(KeyCode::ArrowRight);
    }
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(7, 19), Cell(8, 18), Cell(8, 19), Cell(9, 18)],
            center: (8.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    assert_eq!(&app.obstacles(), &[]);
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(1, 2), Cell(2, 2), Cell(3, 2), Cell(4, 2)],
            center: (2.5, 1.5),
            color: get_tetromino(I).color
        }]
    );
}

#[test]
fn shift5() {
    // shift down even before gravity kicks in, then shift right.
    let mut app = make_test_app([Z, T]);
    app.update();
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 19), Cell(4, 18), Cell(4, 19), Cell(5, 18)],
            center: (4.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    for _ in 0..19 {
        app.release_then_press(KeyCode::ArrowDown);
    }
    app.release_then_press(KeyCode::ArrowRight);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(4, 1), Cell(5, 0), Cell(5, 1), Cell(6, 0)],
            center: (5.0, 0.0),
            color: get_tetromino(Z).color
        }]
    );
    assert_eq!(&app.obstacles(), &[]);
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(2, 3), Cell(1, 2), Cell(2, 2), Cell(3, 2)],
            center: (2.0, 2.0),
            color: get_tetromino(T).color
        }]
    );
}

#[test]
fn z_rotate_hold_arrow() {
    let mut app = make_test_app([Z, I]);
    app.update();
    app.release_then_press(KeyCode::ArrowUp);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(5, 19), Cell(4, 18), Cell(5, 18), Cell(4, 17)],
            center: (4.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    app.update();
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(5, 19), Cell(4, 18), Cell(5, 18), Cell(4, 17)],
            center: (4.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    assert_eq!(&app.obstacles(), &[]);
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(1, 2), Cell(2, 2), Cell(3, 2), Cell(4, 2)],
            center: (2.5, 1.5),
            color: get_tetromino(I).color
        }]
    );
}

#[test]
fn s_rotate() {
    let color = get_tetromino(S).color;
    let mut app = make_test_app([S]);
    test_rotation(
        &mut app,
        KeyCode::KeyS,
        [
            Tetromino {
                cells: [Cell(4, 19), Cell(4, 18), Cell(5, 18), Cell(5, 17)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(5, 18), Cell(4, 18), Cell(4, 17), Cell(3, 17)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(4, 17), Cell(4, 18), Cell(3, 18), Cell(3, 19)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(3, 18), Cell(4, 18), Cell(4, 19), Cell(5, 19)],
                center: (4.0, 18.0),
                color,
            },
        ],
    );
}

#[test]
fn z_rotate() {
    let color = get_tetromino(Z).color;
    let mut app = make_test_app([Z]);
    test_rotation(
        &mut app,
        KeyCode::KeyZ,
        [
            Tetromino {
                cells: [Cell(5, 19), Cell(4, 18), Cell(5, 18), Cell(4, 17)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(5, 17), Cell(4, 18), Cell(4, 17), Cell(3, 18)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(3, 17), Cell(4, 18), Cell(3, 18), Cell(4, 19)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(3, 19), Cell(4, 18), Cell(4, 19), Cell(5, 18)],
                center: (4.0, 18.0),
                color,
            },
        ],
    );
}

#[test]
fn l_rotate() {
    let color = get_tetromino(L).color;
    let mut app = make_test_app([L]);
    test_rotation(
        &mut app,
        KeyCode::KeyL,
        [
            Tetromino {
                cells: [Cell(4, 19), Cell(4, 18), Cell(4, 17), Cell(5, 19)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(5, 18), Cell(4, 18), Cell(3, 18), Cell(5, 17)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(4, 17), Cell(4, 18), Cell(4, 19), Cell(3, 17)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(3, 18), Cell(4, 18), Cell(5, 18), Cell(3, 19)],
                center: (4.0, 18.0),
                color,
            },
        ],
    );
}

#[test]
fn j_rotate() {
    let color = get_tetromino(J).color;
    let mut app = make_test_app([J]);
    let mut ts = [
        Tetromino {
            cells: [Cell(2, 3), Cell(2, 2), Cell(2, 1), Cell(3, 1)],
            center: (4.0, 18.0),
            color,
        },
        Tetromino {
            cells: [Cell(3, 2), Cell(2, 2), Cell(1, 2), Cell(1, 1)],
            center: (4.0, 18.0),
            color,
        },
        Tetromino {
            cells: [Cell(2, 1), Cell(2, 2), Cell(2, 3), Cell(1, 3)],
            center: (4.0, 18.0),
            color,
        },
        Tetromino {
            cells: [Cell(1, 2), Cell(2, 2), Cell(3, 2), Cell(3, 3)],
            center: (4.0, 18.0),
            color,
        },
    ];
    for i in &mut ts {
        i.shift(2, 16);
        i.center = (4.0, 18.0);
    }
    eprintln!("{ts:?}");
    test_rotation(
        &mut app,
        KeyCode::KeyJ,
        [
            Tetromino {
                cells: [Cell(4, 19), Cell(4, 18), Cell(4, 17), Cell(5, 17)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(5, 18), Cell(4, 18), Cell(3, 18), Cell(3, 17)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(4, 17), Cell(4, 18), Cell(4, 19), Cell(3, 19)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(3, 18), Cell(4, 18), Cell(5, 18), Cell(5, 19)],
                center: (4.0, 18.0),
                color,
            },
        ],
    );
}

#[test]
fn t_rotate() {
    let color = get_tetromino(T).color;
    let mut app = make_test_app([T]);
    let mut ts = [
        Tetromino {
            cells: [Cell(3, 2), Cell(2, 3), Cell(2, 2), Cell(2, 1)],
            center: (4.0, 18.0),
            color,
        },
        Tetromino {
            cells: [Cell(2, 1), Cell(3, 2), Cell(2, 2), Cell(1, 2)],
            center: (4.0, 18.0),
            color,
        },
        Tetromino {
            cells: [Cell(1, 2), Cell(2, 1), Cell(2, 2), Cell(2, 3)],
            center: (4.0, 18.0),
            color,
        },
        Tetromino {
            cells: [Cell(2, 3), Cell(1, 2), Cell(2, 2), Cell(3, 2)],
            center: (4.0, 18.0),
            color,
        },
    ];
    for i in &mut ts {
        i.shift(2, 16);
        i.center = (4.0, 18.0);
    }
    eprintln!("{ts:?}");
    test_rotation(
        &mut app,
        KeyCode::KeyT,
        [
            Tetromino {
                cells: [Cell(5, 18), Cell(4, 19), Cell(4, 18), Cell(4, 17)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(4, 17), Cell(5, 18), Cell(4, 18), Cell(3, 18)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(3, 18), Cell(4, 17), Cell(4, 18), Cell(4, 19)],
                center: (4.0, 18.0),
                color,
            },
            Tetromino {
                cells: [Cell(4, 19), Cell(3, 18), Cell(4, 18), Cell(5, 18)],
                center: (4.0, 18.0),
                color,
            },
        ],
    );
}

#[test]
fn i_rotate() {
    let color = get_tetromino(I).color;
    let mut app = make_test_app([I]);
    test_rotation(
        &mut app,
        KeyCode::KeyI,
        [
            Tetromino {
                cells: [Cell(5, 20), Cell(5, 19), Cell(5, 18), Cell(5, 17)],
                center: (4.5, 18.5),
                color,
            },
            Tetromino {
                cells: [Cell(6, 18), Cell(5, 18), Cell(4, 18), Cell(3, 18)],
                center: (4.5, 18.5),
                color,
            },
            Tetromino {
                cells: [Cell(4, 17), Cell(4, 18), Cell(4, 19), Cell(4, 20)],
                center: (4.5, 18.5),
                color,
            },
            Tetromino {
                cells: [Cell(3, 19), Cell(4, 19), Cell(5, 19), Cell(6, 19)],
                center: (4.5, 18.5),
                color,
            },
        ],
    );
}

#[test]
fn o_rotate() {
    let color = get_tetromino(O).color;
    let mut app = make_test_app([O]);
    test_rotation(
        &mut app,
        KeyCode::KeyO,
        [
            Tetromino {
                cells: [Cell(4, 18), Cell(4, 19), Cell(5, 18), Cell(5, 19)],
                center: (4.5, 18.5),
                color,
            },
            Tetromino {
                cells: [Cell(4, 18), Cell(4, 19), Cell(5, 18), Cell(5, 19)],
                center: (4.5, 18.5),
                color,
            },
            Tetromino {
                cells: [Cell(4, 18), Cell(4, 19), Cell(5, 18), Cell(5, 19)],
                center: (4.5, 18.5),
                color,
            },
            Tetromino {
                cells: [Cell(4, 18), Cell(4, 19), Cell(5, 18), Cell(5, 19)],
                center: (4.5, 18.5),
                color,
            },
        ],
    );
}

#[test]
fn shift_and_rotate() {
    // block rotation on board edges (no wall kick)
    let mut app = make_test_app([Z, I]);
    app.update();
    app.release_then_press(KeyCode::ArrowUp);
    app.release_then_press(KeyCode::ArrowUp);
    app.release_then_press(KeyCode::ArrowUp);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(3, 17), Cell(4, 18), Cell(3, 18), Cell(4, 19)],
            center: (4.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    for _ in 0..6 {
        app.release_then_press(KeyCode::ArrowRight);
    }
    app.release_then_press(KeyCode::ArrowUp);
    assert_eq!(
        app.tetrominos::<Active>(),
        vec![Tetromino {
            cells: [Cell(8, 17), Cell(9, 18), Cell(8, 18), Cell(9, 19)],
            center: (9.0, 18.0),
            color: get_tetromino(Z).color
        }]
    );
    assert_eq!(&app.obstacles(), &[]);
    assert_eq!(
        app.tetrominos::<Next>(),
        vec![Tetromino {
            cells: [Cell(1, 2), Cell(2, 2), Cell(3, 2), Cell(4, 2)],
            center: (2.5, 1.5),
            color: get_tetromino(I).color
        }]
    );
}

#[test]
fn gravity_and_input() {
    // drop 1 block 19 times then lock it,
    // after that move a different block 3 steps to the right and lock that too.
    let mut app = make_test_app([J, L, O]);
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
    for i in 0..21 {
        let before = Instant::now();
        if i < 4 {
            // this includes two updates already
            app.release_then_press(KeyCode::ArrowRight);
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
        [Cell(7, 0), Cell(8, 0), Cell(9, 0), Cell(7, 1)].map(|cell| Block {
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
