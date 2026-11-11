//! Game configuration used for testing and user-initiated setup.

use bevy::time::{Timer, TimerMode};
use serde::{Deserialize, Serialize};

use crate::{bag::*, data::*};

/// Game configuration to read from the user or from the tests.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameConfig {
    /// The type of bag for this game.
    pub bag: BagType,

    /// Whether to animate the title text.
    pub animate_title: bool,
}

impl GameConfig {
    /// Build an initial game state based on this configuration.
    pub fn build_game_state(&self) -> GameState {
        GameState {
            manual_drop_gravity: SOFT_DROP_GRAVITY,
            bag: self.bag.create_bag(),
            score: 0,
            lines_cleared: 0,
            lines_cleared_since_last_level: 0,
            level: 0,
            gravity_timer: Timer::new(GameState::initial_drop_interval(), TimerMode::Repeating),
        }
    }

    #[cfg(feature = "config")]
    /// Read a configuration from given JSON data.
    pub fn load(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// What type of bag to create in the initial state.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum BagType {
    /// A deterministic bag that cycles through all tetrominos
    Deterministic,
    #[cfg(feature = "rng")]
    /// A randomized bag with a given starting random seed
    FixedSeed(u64),
    #[cfg(feature = "rng")]
    /// A randomized bag with a seed picked at runtime
    RandomSeed,
}

impl BagType {
    /// Create a new bag based on the parameters specified by this object.
    pub fn create_bag(&self) -> Box<dyn Bag + Sync> {
        use BagType::*;

        match self {
            Deterministic => Box::new(DeterministicBag::default()),
            #[cfg(feature = "rng")]
            FixedSeed(seed) => Box::new(RandomBag::from_seed(*seed)),
            #[cfg(feature = "rng")]
            RandomSeed => Box::new(RandomBag::default()),
        }
    }
}

impl Default for BagType {
    #[cfg(feature = "rng")]
    fn default() -> BagType {
        Self::RandomSeed
    }

    #[cfg(not(feature = "rng"))]
    fn default() -> BagType {
        Self::Deterministic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[test]
    fn default_game_state() {
        let cfg = GameConfig {
            bag: BagType::Deterministic,
            animate_title: true,
        };
        let state = cfg.build_game_state();

        assert_eq!(state.manual_drop_gravity, SOFT_DROP_GRAVITY);
        assert!((state.bag.as_ref() as &dyn Any).is::<DeterministicBag>());
        assert_eq!(state.score, 0);
        assert_eq!(state.lines_cleared, 0);
        assert_eq!(state.lines_cleared_since_last_level, 0);
        assert_eq!(state.level, 0);
        assert_eq!(
            state.gravity_timer.duration(),
            GameState::initial_drop_interval()
        );
        assert_eq!(state.gravity_timer.mode(), TimerMode::Repeating);
    }

    #[test]
    #[cfg(feature = "rng")]
    fn bag_creation() {
        let cfg = GameConfig {
            bag: BagType::Deterministic,
            animate_title: true,
        };
        let state = cfg.build_game_state();

        assert!((state.bag.as_ref() as &dyn Any).is::<DeterministicBag>());

        let cfg = GameConfig {
            bag: BagType::FixedSeed(727),
            animate_title: true,
        };
        let state = cfg.build_game_state();

        assert!((state.bag.as_ref() as &dyn Any).is::<RandomBag>());
        assert_eq!(
            (state.bag.as_ref() as &dyn Any).downcast_ref::<RandomBag>(),
            Some(&RandomBag::from_seed(727))
        );

        let cfg = GameConfig {
            bag: BagType::RandomSeed,
            animate_title: true,
        };
        let state1 = cfg.build_game_state();
        let state2 = cfg.build_game_state();

        assert!((state1.bag.as_ref() as &dyn Any).is::<RandomBag>());
        assert!((state2.bag.as_ref() as &dyn Any).is::<RandomBag>());
        assert_ne!(
            (state1.bag.as_ref() as &dyn Any).downcast_ref::<RandomBag>(),
            (state2.bag.as_ref() as &dyn Any).downcast_ref::<RandomBag>()
        );
    }

    // From: Claude Code (AI-generated), tests for GameConfig::load
    #[cfg(feature = "config")]
    mod load {
        use super::*;

        fn config(bag: &str, animate_title: &str) -> String {
            format!(r#"{{"bag": {bag}, "animate_title": {animate_title}}}"#)
        }

        #[test]
        fn deterministic_bag() {
            let cfg = GameConfig::load(&config(r#""Deterministic""#, "true")).unwrap();
            assert_eq!(cfg.bag, BagType::Deterministic);
        }

        #[test]
        fn animate_title_on_and_off() {
            [true, false].into_iter().for_each(|animate_title| {
                let json = config(r#""Deterministic""#, &animate_title.to_string());
                assert_eq!(
                    GameConfig::load(&json).unwrap(),
                    GameConfig {
                        bag: BagType::Deterministic,
                        animate_title,
                    }
                );
            });
        }

        #[test]
        fn ignores_whitespace_and_field_order() {
            let json = "  {\n\t\"animate_title\" : false ,\n\t\"bag\" : \"Deterministic\"\n}  ";
            assert_eq!(
                GameConfig::load(json).unwrap(),
                GameConfig {
                    bag: BagType::Deterministic,
                    animate_title: false,
                }
            );
        }

        #[test]
        fn round_trips_through_serialization() {
            let cfg = GameConfig {
                bag: BagType::Deterministic,
                animate_title: false,
            };
            let json = serde_json::to_string(&cfg).unwrap();
            assert_eq!(GameConfig::load(&json).unwrap(), cfg);
        }

        #[test]
        fn rejects_unknown_bag_type() {
            assert!(GameConfig::load(&config(r#""Shuffled""#, "true")).is_err());
        }

        #[test]
        fn rejects_missing_fields() {
            assert!(GameConfig::load(r#"{"animate_title": true}"#).is_err());
            assert!(GameConfig::load(r#"{"bag": "Deterministic"}"#).is_err());
            assert!(GameConfig::load("{}").is_err());
        }

        #[test]
        fn rejects_wrongly_typed_fields() {
            assert!(GameConfig::load(&config(r#""Deterministic""#, r#""yes""#)).is_err());
            assert!(GameConfig::load(&config("3", "true")).is_err());
        }

        #[test]
        fn rejects_malformed_json() {
            ["", "not json", r#"{"bag": "Deterministic""#, "[]"]
                .into_iter()
                .for_each(|json| assert!(GameConfig::load(json).is_err(), "accepted {json:?}"));
        }

        #[test]
        fn loads_the_provided_deterministic_config() {
            let path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_data/config-deterministic.json"
            );
            let cfg = GameConfig::load(&std::fs::read_to_string(path).unwrap()).unwrap();
            assert_eq!(
                cfg,
                GameConfig {
                    bag: BagType::Deterministic,
                    animate_title: true,
                }
            );
        }

        #[test]
        #[cfg(feature = "rng")]
        fn fixed_seed_bag() {
            let cfg = GameConfig::load(&config(r#"{"FixedSeed": 727}"#, "true")).unwrap();
            assert_eq!(cfg.bag, BagType::FixedSeed(727));
        }

        #[test]
        #[cfg(feature = "rng")]
        fn random_seed_bag() {
            let cfg = GameConfig::load(&config(r#""RandomSeed""#, "false")).unwrap();
            assert_eq!(
                cfg,
                GameConfig {
                    bag: BagType::RandomSeed,
                    animate_title: false,
                }
            );
        }

        #[test]
        #[cfg(feature = "rng")]
        fn rejects_invalid_seeds() {
            [
                r#"{"FixedSeed": -1}"#,
                r#"{"FixedSeed": 1.5}"#,
                r#"{"FixedSeed": "abc"}"#,
                r#"{"FixedSeed": 18446744073709551616}"#,
                r#""FixedSeed""#,
            ]
            .into_iter()
            .for_each(|bag| {
                assert!(
                    GameConfig::load(&config(bag, "true")).is_err(),
                    "accepted {bag}"
                )
            });
        }

        #[test]
        #[cfg(feature = "rng")]
        fn loads_the_provided_seeded_configs() {
            [
                ("config-0.json", 0),
                ("config-67.json", 67),
                ("config-727.json", 727),
            ]
            .into_iter()
            .for_each(|(file, seed)| {
                let path = format!("{}/test_data/{file}", env!("CARGO_MANIFEST_DIR"));
                let cfg = GameConfig::load(&std::fs::read_to_string(path).unwrap()).unwrap();
                assert_eq!(cfg.bag, BagType::FixedSeed(seed));
            });
        }
    }
}
