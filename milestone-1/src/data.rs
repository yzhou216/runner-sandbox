//! Data structures and fixtures used by the ECS framework

use serde::{Deserialize, Serialize};
use std::time::Duration;

use bevy::{color::palettes::tailwind, prelude::*};

use crate::bag::Bag;

/// A 2-d cell, represented via its x and y coordinates
#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cell(pub i32, pub i32);
impl Cell {
    /// Rotate this cell 90 degrees clockwise around the given center point.
    ///
    /// The result is rounded to the nearest integer, with ties broken away from zero.
    fn rotate_90_deg_cw(&self, center_x: f32, center_y: f32) -> Cell {
        Cell(
            (self.1 as f32 - center_y + center_x).round() as i32,
            (-self.0 as f32 + center_x + center_y).round() as i32,
        )
    }

    /// Check whether this cell is in bounds.
    ///
    /// This takes the invisible rows into account
    pub fn in_bounds(&self) -> bool {
        (0..BOARD_WIDTH as i32).contains(&self.0)
            && (0..BOARD_HEIGHT as i32 + INVISIBLE_ROWS as i32).contains(&self.1)
    }

    /// Check whether this cell is in the visible part of the board.
    pub fn is_visible(&self) -> bool {
        (0..BOARD_WIDTH as i32).contains(&self.0) && (0..BOARD_HEIGHT as i32).contains(&self.1)
    }
}

/// A bounding box for a tetromino
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Bounds {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

/// Whether this tetromino is the active one
#[derive(Component, Copy, Clone)]
pub struct Active;

/// Whether this tetromino is the next one
#[derive(Component, Copy, Clone)]
pub struct Next;

/// Whether a block is an obstacle
#[derive(Component)]
pub struct Obstacle;

/// The cell associated with a tetromino
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct Tetromino {
    pub cells: [Cell; 4],
    pub center: (f32, f32),
    pub color: Color,
}

impl Tetromino {
    /// Cells of the tetromino
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Center of the tetromino for the purposes of rotation
    pub fn center(&self) -> (f32, f32) {
        self.center
    }

    /// The bounding box of the tetromino
    pub fn bounds(&self) -> Bounds {
        debug_assert!(!self.cells.is_empty());
        Bounds {
            left: self.cells.iter().map(|c| c.0).min().unwrap(),
            right: self.cells.iter().map(|c| c.0).max().unwrap(),
            bottom: self.cells.iter().map(|c| c.1).min().unwrap(),
            top: self.cells.iter().map(|c| c.1).max().unwrap(),
        }
    }

    /// Is the tetromino in bounds?
    pub fn in_bounds(&self) -> bool {
        self.cells.iter().all(Cell::in_bounds)
    }

    /// Is this the O tetromino?
    pub fn is_o(&self) -> bool {
        let mut cells = self.cells;
        cells.sort();
        let Cell(x, y) = cells[0];
        cells
            == [
                Cell(x, y),
                Cell(x, y + 1),
                Cell(x + 1, y),
                Cell(x + 1, y + 1),
            ]
    }

    /// Rotate this tetromino 90 degrees clockwise.
    pub fn rotate(&mut self) {
        if self.is_o() {
            return;
        }

        let (center_x, center_y) = self.center;
        self.cells = self
            .cells
            .map(|cell| cell.rotate_90_deg_cw(center_x, center_y));
    }

    /// Shift all the cells in the tetromino by the given amount
    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.center = (self.center.0 + dx as f32, self.center.1 + dy as f32);
        self.cells = self.cells.map(|Cell(x, y)| Cell(x + dx, y + dy));
    }
}

// Custom equality information that ignores the ordering of cells
impl PartialEq for Tetromino {
    fn eq(&self, other: &Self) -> bool {
        let mut cells1 = self.cells;
        cells1.sort();
        let mut cells2 = other.cells;
        cells2.sort();
        cells1 == cells2 && self.center == other.center && self.color == other.color
    }
}

/// Height of the visible part of the board
pub const BOARD_HEIGHT: u32 = 20;
/// Width of the visible part of the board
pub const BOARD_WIDTH: u32 = 10;
/// The number of invisible rows on top of the board
pub const INVISIBLE_ROWS: u32 = 3;
/// Gravity number to be used when hard drop is enabled
pub const HARD_DROP_GRAVITY: u32 = 20;
/// Gravity number to be used when hard drop is disabled
pub const SOFT_DROP_GRAVITY: u32 = 1;

/// Global game configuration.
#[derive(Resource)]
pub struct GameState {
    /// Total score so far
    pub score: u32,
    /// Total lines cleared
    pub lines_cleared: u32,
    /// Lines cleared since last level up
    pub lines_cleared_since_last_level: u32,
    /// The next piece bag
    pub bag: Box<dyn Bag + Sync>,
    /// Current level
    pub level: u32,
    /// How many blocks to drop on user input.  This value should change
    /// depending on whether hard drop is enabled.
    pub manual_drop_gravity: u32,
    /// Timer for querying when the next automatic drop happens
    pub gravity_timer: Timer,
}

impl GameState {
    /// Maximum level for gravity changes
    pub const MAX_LEVEL: usize = 29;
    /// Framerate of the game for the purposes of gravity calculations
    pub const FRAMERATE: f32 = 60.0;
    /// Gravity block drop intervals for each level
    pub const INTERVALS: [f32; Self::MAX_LEVEL] = [
        48.0, 43.0, 38.0, 33.0, 28.0, 23.0, 18.0, 13.0, 8.0, 6.0, 5.0, 5.0, 5.0, 4.0, 4.0, 4.0,
        3.0, 3.0, 3.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 1.0,
    ];

    /// Current score
    pub fn score(&self) -> u32 {
        self.score
    }

    /// Current level
    pub fn level(&self) -> u32 {
        self.level
    }

    /// Auto-drop interval (i.e., gravity)
    pub fn drop_interval(&self) -> Duration {
        // Levels past the end of the table keep the fastest speed
        let level = (self.level as usize).min(Self::MAX_LEVEL - 1);
        Duration::from_secs_f32(Self::INTERVALS[level] / Self::FRAMERATE)
    }

    /// The drop interval in the beginning of the game.
    pub const fn initial_drop_interval() -> Duration {
        Duration::from_millis((Self::INTERVALS[0] / Self::FRAMERATE * 1000.0) as u64)
    }
}

/// Different types of canonical tetrominos.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum TetrominoType {
    S,
    Z,
    L,
    J,
    T,
    O,
    I,
}

/// Get a fresh canonical tetromino of the given type.
pub const fn get_tetromino(typ: TetrominoType) -> Tetromino {
    use TetrominoType::*;
    let (cells, center, color) = match typ {
        S => (
            [Cell(-1, 0), Cell(0, 0), Cell(0, 1), Cell(1, 1)],
            (0., 0.),
            tailwind::RED_400,
        ),
        Z => (
            [Cell(-1, 1), Cell(0, 0), Cell(0, 1), Cell(1, 0)],
            (0., 0.),
            tailwind::ORANGE_400,
        ),
        L => (
            [Cell(-1, 0), Cell(0, 0), Cell(1, 0), Cell(-1, 1)],
            (0., 0.),
            tailwind::YELLOW_400,
        ),
        J => (
            [Cell(-1, 0), Cell(0, 0), Cell(1, 0), Cell(1, 1)],
            (0., 0.),
            tailwind::GREEN_400,
        ),
        T => (
            [Cell(0, 1), Cell(-1, 0), Cell(0, 0), Cell(1, 0)],
            (0.0, 0.0),
            tailwind::BLUE_400,
        ),
        O => (
            [Cell(0, 0), Cell(0, 1), Cell(1, 0), Cell(1, 1)],
            (0.5, 0.5),
            tailwind::CYAN_400,
        ),
        I => (
            [Cell(-1, 0), Cell(0, 0), Cell(1, 0), Cell(2, 0)],
            (0.5, -0.5),
            tailwind::PURPLE_400,
        ),
    };

    Tetromino {
        cells,
        center,
        color: Color::Srgba(color),
    }
}

/// All tetrominos that appear in the game in a fixed order.
pub const ALL_TETROMINO_TYPES: [TetrominoType; 7] = [
    TetrominoType::S,
    TetrominoType::Z,
    TetrominoType::L,
    TetrominoType::J,
    TetrominoType::T,
    TetrominoType::O,
    TetrominoType::I,
];

#[cfg(test)]
mod tests {
    use crate::bag::DeterministicBag;

    use super::*;

    // Create a game state fixture
    #[allow(dead_code)]
    fn mk_game_state() -> GameState {
        GameState {
            score: 0,
            lines_cleared: 0,
            lines_cleared_since_last_level: 0,
            bag: Box::new(DeterministicBag::default()),
            level: 0,
            manual_drop_gravity: 0,
            gravity_timer: Timer::new(Duration::from_millis(1000), TimerMode::Repeating),
        }
    }

    #[test]
    fn rotate_cell() {
        let cell = Cell(3, 4);
        assert_eq!(Cell(4, -3), cell.rotate_90_deg_cw(0.0, 0.0));
        assert_eq!(Cell(4, 7), cell.rotate_90_deg_cw(5.0, 5.0));
        assert_eq!(Cell(3, 7), cell.rotate_90_deg_cw(4.5, 5.5));

        let cell = Cell(-2, 4);
        assert_eq!(Cell(4, 2), cell.rotate_90_deg_cw(0.0, 0.0));

        let cell = Cell(2, -4);
        assert_eq!(Cell(-4, -2), cell.rotate_90_deg_cw(0.0, 0.0));
    }

    #[test]
    fn is_o() {
        for typ in ALL_TETROMINO_TYPES {
            assert_eq!(get_tetromino(typ).is_o(), matches!(typ, TetrominoType::O));
        }
    }

    #[test]
    fn rotate_cells() {
        use TetrominoType::*;

        let mut tetromino = get_tetromino(S);
        tetromino.rotate();
        assert_eq!(
            tetromino.cells,
            [Cell(0, 1), Cell(0, 0), Cell(1, 0), Cell(1, -1)]
        );
        tetromino.rotate();
        assert_eq!(
            tetromino.cells,
            [Cell(1, 0), Cell(0, 0), Cell(0, -1), Cell(-1, -1)]
        );
        tetromino.rotate();
        assert_eq!(
            tetromino.cells,
            [Cell(0, -1), Cell(0, 0), Cell(-1, 0), Cell(-1, 1)]
        );
        tetromino.rotate();
        assert_eq!(tetromino, get_tetromino(S));

        let mut tetromino = get_tetromino(Z);
        tetromino.rotate();
        assert_eq!(
            tetromino.cells,
            [Cell(1, 1), Cell(0, 0), Cell(1, 0), Cell(0, -1)]
        );

        let mut tetromino = get_tetromino(L);
        tetromino.rotate();
        assert_eq!(
            tetromino.cells,
            [Cell(0, 1), Cell(0, 0), Cell(0, -1), Cell(1, 1)]
        );

        let mut tetromino = get_tetromino(J);
        tetromino.rotate();
        assert_eq!(
            tetromino.cells,
            [Cell(0, 1), Cell(0, 0), Cell(0, -1), Cell(1, -1)]
        );

        let mut tetromino = get_tetromino(T);
        tetromino.rotate();
        assert_eq!(
            tetromino.cells,
            [Cell(1, 0), Cell(0, 1), Cell(0, 0), Cell(0, -1)]
        );

        let mut tetromino = get_tetromino(O);
        tetromino.rotate();
        assert_eq!(
            tetromino.cells,
            [Cell(0, 0), Cell(0, 1), Cell(1, 0), Cell(1, 1)]
        );

        let mut tetromino = get_tetromino(I);
        tetromino.rotate();
        assert_eq!(
            tetromino.cells,
            [Cell(1, 1), Cell(1, 0), Cell(1, -1), Cell(1, -2)]
        );
    }

    #[test]
    fn rotate_semi_involution() {
        for typ in ALL_TETROMINO_TYPES {
            let mut tetromino = get_tetromino(typ);
            tetromino.rotate();
            tetromino.rotate();
            tetromino.rotate();
            tetromino.rotate();
            assert_eq!(tetromino, get_tetromino(typ));
        }
    }

    #[test]
    fn rotate_test_center() {
        for typ in ALL_TETROMINO_TYPES {
            let mut tetromino = get_tetromino(typ);
            let old_tetromino = tetromino;
            tetromino.rotate();
            assert_eq!(tetromino.center, old_tetromino.center);
            let old_tetromino = tetromino;
            tetromino.rotate();
            assert_eq!(tetromino.center, old_tetromino.center);
            let old_tetromino = tetromino;
            tetromino.rotate();
            assert_eq!(tetromino.center, old_tetromino.center);
            let old_tetromino = tetromino;
            tetromino.rotate();
            assert_eq!(tetromino.center, old_tetromino.center);
        }
    }

    #[test]
    fn shift() {
        let mut tetromino = get_tetromino(TetrominoType::L);
        tetromino.shift(3, -2);
        assert_eq!(
            tetromino,
            Tetromino {
                cells: [Cell(2, -2), Cell(3, -2), Cell(4, -2), Cell(2, -1)],
                center: (3.0, -2.0),
                color: Color::from(tailwind::YELLOW_400),
            }
        );
    }

    #[test]
    fn initial_drop_interval() {
        assert_eq!(
            GameState::initial_drop_interval(),
            Duration::from_millis(800)
        );
    }

    #[test]
    #[cfg(feature = "score")]
    fn drop_interval() {
        let mut state = mk_game_state();
        for (level, frames) in GameState::INTERVALS.into_iter().enumerate() {
            state.level = level as u32;
            assert_eq!(
                state.drop_interval().as_secs_f32(),
                frames / GameState::FRAMERATE
            );
        }
    }
}
