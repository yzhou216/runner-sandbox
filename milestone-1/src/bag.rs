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
    use rand::{SeedableRng, rngs::SmallRng, seq::SliceRandom};

    /// The random tile bag
    #[derive(PartialEq, Debug)]
    pub struct RandomBag {
        remaining_pieces: Vec<Tetromino>,
        rng: SmallRng,
    }

    impl RandomBag {
        /// Create a bag from given starting RNG seed.
        pub fn from_seed(seed: u64) -> Self {
            Self {
                remaining_pieces: vec![],
                rng: SmallRng::seed_from_u64(seed),
            }
        }

        // Refill the bag if it is empty.  This should create one of each
        // tetromino, shuffle them, and put them in the bag.
        fn refill(&mut self) {
            debug_assert!(self.remaining_pieces.is_empty());
            self.remaining_pieces = ALL_TETROMINO_TYPES.map(get_tetromino).to_vec();
            self.remaining_pieces.shuffle(&mut self.rng);
        }
    }

    impl Bag for RandomBag {
        fn next_tetromino(&mut self) -> Tetromino {
            let next = self.peek();
            self.remaining_pieces.pop();
            next
        }

        fn peek(&mut self) -> Tetromino {
            if self.remaining_pieces.is_empty() {
                self.refill();
            }
            *self
                .remaining_pieces
                .last()
                .expect("the bag was refilled when it ran empty")
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

// From: Claude Code (AI-generated), tests for RandomBag
#[cfg(all(test, feature = "rng"))]
mod rng_tests {
    use super::*;

    fn draw(bag: &mut RandomBag, count: usize) -> Vec<Tetromino> {
        (0..count).map(|_| bag.next_tetromino()).collect()
    }

    #[test]
    fn every_bag_has_each_tetromino_once() {
        (0..32).for_each(|seed| {
            draw(&mut RandomBag::from_seed(seed), 21)
                .chunks(7)
                .for_each(|bag| {
                    ALL_TETROMINO_TYPES.iter().for_each(|&typ| {
                        let copies = bag.iter().filter(|&&t| t == get_tetromino(typ)).count();
                        assert_eq!(copies, 1, "seed {seed} does not have one of each piece");
                    });
                });
        });
    }

    #[test]
    fn peek_agrees_with_the_next_tetromino() {
        let mut bag = RandomBag::from_seed(5);
        (0..20).for_each(|_| {
            let peeked = bag.peek();
            assert_eq!(peeked, bag.peek(), "peeking must not take the piece");
            assert_eq!(peeked, bag.next_tetromino());
        });
    }

    #[test]
    fn the_same_seed_gives_the_same_pieces() {
        let first = draw(&mut RandomBag::from_seed(42), 30);
        let second = draw(&mut RandomBag::from_seed(42), 30);
        assert_eq!(first, second);
    }

    #[test]
    fn different_seeds_give_different_pieces() {
        let sequences: Vec<_> = (0..8)
            .map(|seed| {
                draw(&mut RandomBag::from_seed(seed), 14)
                    .iter()
                    .map(|t| format!("{:?}", t.cells()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(sequences.windows(2).any(|pair| pair[0] != pair[1]));
    }
}
