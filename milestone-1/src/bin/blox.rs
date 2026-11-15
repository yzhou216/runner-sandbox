use std::path::PathBuf;

use bevy::prelude::*;
use blox::{
    build_app,
    config::{BagType, GameConfig},
    rr::{
        GameRecording, RecordingOutput, RecordingPlugin, ReplayPlugin,
        test_replay::TestReplayPlugin,
    },
};

use clap::Parser;

/// A simple block dropping game.
#[derive(Parser)]
struct CLIOptions {
    /// Configuration file to load.
    #[cfg(feature = "config")]
    cfg: Option<PathBuf>,

    /// Record inputs to given file
    #[arg(long, value_name = "FILE")]
    record: Option<PathBuf>,

    /// Replay given recording
    #[arg(long, value_name = "FILE")]
    replay: Option<PathBuf>,

    /// Replay given recording, checking against the recorded state stream
    #[arg(long, value_name = "FILE", requires = "check_scores")]
    test: Option<PathBuf>,

    /// Whether the test case should check for scores/levels/etc.
    #[arg(long, requires = "test")]
    check_scores: Option<bool>,
}

impl CLIOptions {
    #[cfg(feature = "config")]
    fn read_config(&self) -> Result<GameConfig, Box<dyn std::error::Error>> {
        Ok(if let Some(cfg) = &self.cfg {
            GameConfig::load(&String::from_utf8(std::fs::read(cfg)?)?)?
        } else {
            Self::default_config()
        })
    }

    #[cfg(not(feature = "config"))]
    fn read_config(&self) -> Result<GameConfig, Box<dyn std::error::Error>> {
        Ok(Self::default_config())
    }

    fn default_config() -> GameConfig {
        GameConfig {
            bag: BagType::default(),
            animate_title: true,
        }
    }
}

fn main() {
    let args = CLIOptions::parse();
    let cfg = args.read_config().expect("Could not read the config file");

    let enabled = [
        args.record.is_some(),
        args.replay.is_some(),
        args.test.is_some(),
    ];

    assert!(
        enabled.map(|b| b as u32).into_iter().sum::<u32>() <= 1,
        "Can record, replay or test but cannot do multiple of those simultaneously.",
    );

    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    build_app(&mut app, cfg);

    if let Some(f) = args.record {
        app.insert_resource(RecordingOutput(f))
            .add_plugins(RecordingPlugin);
    } else if let Some(f) = args.replay {
        let recording: GameRecording =
            serde_json::from_slice(&std::fs::read(f).expect("Cannot read the recording file"))
                .expect("The recording is ill-formatted");
        app.insert_resource(recording).add_plugins(ReplayPlugin);
    } else if let Some(f) = args.test {
        let recording: GameRecording =
            serde_json::from_slice(&std::fs::read(f).expect("Cannot read the recording file"))
                .expect("The recording is ill-formatted");
        app.insert_resource(recording)
            .add_plugins(TestReplayPlugin {
                check_scores: args.check_scores.unwrap(),
            });
    }

    app.run();
}
