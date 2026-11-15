//! Next piece bag used for testing.

use std::collections::VecDeque;

use blox::{
    bag::*,
    data::{Tetromino, TetrominoType, get_tetromino},
};

/// A bag that goes through the specified tiles before switching the
/// control over to the deterministic bag.
#[derive(Debug)]
pub struct TestBag {
    tetrominos: VecDeque<Tetromino>,
    fallback: DeterministicBag,
}

impl TestBag {
    /// Initialize the bag with the given contents.
    pub fn new<T: IntoIterator<Item = TetrominoType>>(content: T) -> Self {
        TestBag {
            tetrominos: content.into_iter().map(get_tetromino).collect(),
            fallback: DeterministicBag::default(),
        }
    }
}

impl Bag for TestBag {
    fn next_tetromino(&mut self) -> Tetromino {
        self.tetrominos
            .pop_front()
            .unwrap_or_else(|| self.fallback.next_tetromino())
    }

    fn peek(&mut self) -> Tetromino {
        self.tetrominos
            .front()
            .cloned()
            .unwrap_or_else(|| self.fallback.peek())
    }
}
