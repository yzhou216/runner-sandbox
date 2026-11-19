//! Next-tile bag implementations

use std::any::Any;

use crate::data::*;

#[cfg(feature = "rng")]
pub use random::*;

/// A generic interface for the next tile bag
pub trait Bag: Send + Any {
    /// Take the next tetromino out of the bag
    fn next_tetromino(&mut self) -> Tetromino;
    /// Peek at the next tetromino, do not take it out.
    ///
    /// This function takes `self` as mutable in case we need to refill the bag.
    fn peek(&mut self) -> Tetromino;
}

/// A deterministic tile bag that cycles between tetrominos.
#[derive(Default, Debug)]
pub struct DeterministicBag {
    next_tile_index: usize,
}

impl Bag for DeterministicBag {
    /// Take the next tetromino out of the bag.
    fn next_tetromino(&mut self) -> Tetromino {
        let t = self.peek();
        self.next_tile_index = (self.next_tile_index + 1) % ALL_TETROMINO_TYPES.len();
        t
    }

    /// Peek at the next tetromino.
    ///
    /// This function is `&mut self` in case the bag needs to be refilled.
    fn peek(&mut self) -> Tetromino {
        get_tetromino(ALL_TETROMINO_TYPES[self.next_tile_index])
    }
}

#[cfg(feature = "rng")]
mod random {
    use super::*;
    use rand::{SeedableRng, rngs::SmallRng};

    /// The random tile bag
    #[derive(PartialEq, Debug)]
    pub struct RandomBag {
        remaining_pieces: Vec<Tetromino>,
        rng: SmallRng,
    }

    impl RandomBag {
        /// Create a bag from given starting RNG seed.
        pub fn from_seed(_seed: u64) -> Self {
            todo!("Create an empty bag, seed the RNG from the given value.")
        }

        // Refill the bag if it is empty.  This should create one of each
        // tetromino, shuffle them, and put them in the bag.
        fn refill(&mut self) {
            debug_assert!(self.remaining_pieces.is_empty());
            todo!()
        }
    }

    impl Bag for RandomBag {
        fn next_tetromino(&mut self) -> Tetromino {
            todo!("Get the next tetromino from the bag.  Refill it if necessary")
        }

        fn peek(&mut self) -> Tetromino {
            todo!()
        }
    }

    impl Default for RandomBag {
        fn default() -> Self {
            Self {
                remaining_pieces: vec![],
                rng: SmallRng::from_os_rng(),
            }
        }
    }
}
