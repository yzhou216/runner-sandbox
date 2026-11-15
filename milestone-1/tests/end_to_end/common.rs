#![allow(dead_code)]

use std::collections::HashSet;
use std::mem;
use std::time::Duration;

use bevy::{
    app::SubApps,
    input::*,
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
    window::*,
    winit::WinitPlugin,
};
use blox::PreGame;
use blox::bag::Bag;
use blox::board::Block;
use blox::config::GameConfig;
use blox::data::*;
use blox::{Game, board::GameOver};
use blox::{PostGame, rr::test_replay::*};

pub use crate::test_bag::*;

const TEST_DATA_DIR: &str = "test_data";

// A resource that is added when the game over event is triggered.
#[derive(Resource)]
pub struct GameIsOver;

#[derive(Resource, Default)]
pub struct KeySequence {
    pub last_frame: HashSet<KeyCode>,
    pub this_frame: HashSet<KeyCode>,
}

impl KeySequence {
    fn update_delta(&mut self, keys: &[KeyCode]) {
        self.last_frame = mem::replace(&mut self.this_frame, keys.iter().cloned().collect());
    }
}

pub fn simulate_key_presses(mut keyboard: ResMut<ButtonInput<KeyCode>>, keys: ResMut<KeySequence>) {
    for k in &keys.last_frame {
        if !keys.this_frame.contains(k) {
            keyboard.release(*k);
        }
    }

    for k in &keys.this_frame {
        keyboard.press(*k);
    }
}

pub(super) fn observe_game_over(_: On<GameOver>, mut commands: Commands) {
    commands.insert_resource(GameIsOver);
}

pub struct Headless(pub SubApps);
impl Headless {
    pub fn new(cfg: GameConfig, bag: Option<Box<dyn Bag + Sync>>) -> Self {
        let mut state = cfg.build_game_state();
        if let Some(bag) = bag {
            state.bag = bag;
        }
        let window_plugin = WindowPlugin {
            primary_window: None,
            exit_condition: ExitCondition::DontExit,
            ..default()
        };

        let mut app = App::new();
        app.add_plugins(DefaultPlugins.set(window_plugin).disable::<WinitPlugin>())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(KeySequence::default());
        blox::build_app(&mut app, cfg);
        // overwrite the default game state
        app.insert_resource(state);
        app.add_systems(Update, simulate_key_presses.in_set(PreGame));
        app.configure_sets(Update, (PreGame, Game, PostGame).chain());
        app.configure_sets(FixedUpdate, (PreGame, Game, PostGame).chain());
        app.finish();
        app.cleanup();
        //NOTE: This is the way to get an app when we want to run it manually.
        //Source: Bevy Examples
        Self(std::mem::take(app.sub_apps_mut()))
    }

    pub fn pressed_keys(&mut self, keys: &[KeyCode]) {
        self.0
            .main
            .world_mut()
            .resource_mut::<KeySequence>()
            .update_delta(keys);
    }

    /// Release all keys, then press the given key.  This method performs
    /// multiple updates.
    pub fn release_then_press(&mut self, key: KeyCode) {
        self.pressed_keys(&[]);
        self.update();
        self.pressed_keys(&[key]);
        self.update();
    }

    /// Run a single world update, does not advance the virtual clock
    pub fn update(&mut self) {
        self.0.update();
    }

    /// Set the relative speed of the virtual clock
    pub fn set_relative_speed(&mut self, multiplier: f32) {
        self.app_mut()
            .world_mut()
            .resource_mut::<Time<Virtual>>()
            .set_relative_speed(multiplier);
    }

    /// Set the relative speed of the virtual clock
    #[allow(dead_code)]
    pub fn relative_speed(&mut self) -> f32 {
        self.app_mut()
            .world_mut()
            .resource::<Time<Virtual>>()
            .relative_speed()
    }

    /// Advance virtual time
    #[allow(dead_code)]
    pub fn advance_time_by(&mut self, duration: Duration) {
        self.app_mut()
            .world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_by(duration);
    }

    /// Set virtual time
    #[allow(dead_code)]
    pub fn advance_time_to(&mut self, duration: Duration) {
        self.app_mut()
            .world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_to(duration);
        self.app_mut()
            .world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_to(duration);
    }

    #[allow(dead_code)]
    pub fn wait_for_render(&mut self) {
        self.0
            .main
            .world()
            .resource::<RenderDevice>()
            .wgpu_device()
            .poll(PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .unwrap();
    }

    pub fn app_mut(&mut self) -> &mut SubApp {
        &mut self.0.main
    }

    pub fn tetrominos<Marker: Component>(&mut self) -> Vec<Tetromino> {
        let world = self.app_mut().world_mut();
        world
            .query::<(&Tetromino, &Marker)>()
            .iter(world)
            .map(|p| *p.0)
            .collect()
    }

    pub fn obstacles(&mut self) -> Vec<Block> {
        let world = self.app_mut().world_mut();
        world
            .query::<(&Block, &Obstacle)>()
            .iter(world)
            .map(|p| *p.0)
            .collect()
    }

    pub fn get_resource<R: Resource>(&mut self) -> Option<&R> {
        let world = self.app_mut().world_mut();
        world.get_resource::<R>()
    }
}

// Section: running external tests

/// Store whether a test passes or not.
#[derive(Resource)]
pub struct TestVerdict(bool);

fn on_test_pass(_: On<TestPass>, mut commands: Commands) {
    commands.insert_resource(TestVerdict(true));
}

fn on_test_fail(_: On<TestFail>, mut commands: Commands) {
    commands.insert_resource(TestVerdict(false));
}

#[cfg(feature = "config")]
pub fn run_recorded_test(recording_file: &str, config_file: &str, check_scores: bool) {
    use std::path::PathBuf;

    let recording_file: PathBuf = [TEST_DATA_DIR, recording_file].iter().collect();
    let config_file: PathBuf = [TEST_DATA_DIR, config_file].iter().collect();

    use blox::rr::GameRecording;
    let config = GameConfig::load(
        &String::from_utf8(std::fs::read(config_file).expect("Cannot read the config file"))
            .expect("The config file is not valid UTF-8"),
    )
    .expect("The config file is ill-formatted");

    let recording: GameRecording = serde_json::from_slice(
        &std::fs::read(recording_file).expect("Cannot read the recording file"),
    )
    .expect("The recording is ill-formatted");

    let last_frame = (recording
        .events
        .iter()
        .last()
        .expect("The event sequence is empty")
        .time
        .as_secs_f32()
        * 64.0)
        .ceil() as usize;

    let mut app = Headless::new(config, None);
    app.app_mut()
        .insert_resource(recording)
        .add_plugins(TestReplayPlugin { check_scores });

    let world = app.app_mut().world_mut();
    world.add_observer(on_test_fail);
    world.add_observer(on_test_pass);

    for _ in 0..(last_frame + 1) {
        app.update();
    }

    assert!(
        app.get_resource::<TestVerdict>()
            .expect("The test did not reach a verdict")
            .0,
        "The recorded test failed, see the logs."
    );
}
