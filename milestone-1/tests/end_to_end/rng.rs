//! Tests involing randomized bags.

use blox::{
    bag::{Bag, RandomBag},
    data::*,
};

fn test_rng_bag(mut bag: RandomBag, expected_tetrominos: &[TetrominoType]) {
    for (i, expected) in expected_tetrominos.iter().enumerate() {
        let peeked = bag.peek();
        let actual = bag.next_tetromino();
        assert_eq!(peeked, actual, "peek and next_tetromino must agree.");
        assert_eq!(
            get_tetromino(*expected),
            actual,
            "expected tetromino #{i} is not the same as the actual tetromino"
        );
    }
}

#[test]
fn random_bag_impl1() {
    use TetrominoType::*;
    test_rng_bag(
        RandomBag::from_seed(727),
        &[O, J, L, Z, I, T, S, J, T, O, Z, S, I, L],
    );
}

#[test]
fn random_bag_impl2() {
    use TetrominoType::*;
    test_rng_bag(
        RandomBag::from_seed(67),
        &[S, J, L, T, I, Z, O, O, J, Z, S, T, I, L],
    );
}

#[test]
fn random_bag_impl3() {
    use TetrominoType::*;
    test_rng_bag(
        RandomBag::from_seed(0),
        &[L, Z, I, T, O, S, J, O, L, T, J, Z, S, I],
    );
}
