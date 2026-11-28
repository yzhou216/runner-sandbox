// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the settings of the configuration file and where it lies

use std::{fs, io, path::Path};

use tempfile::TempDir;

use super::*;
use crate::back::READY_AHEAD;

#[test]
fn file_is_in_the_directory_named_on_the_command_line_or_else_that_of_the_platform() {
    assert_eq!(
        config::file(None),
        dirs::config_dir().map(|directory| directory.join("cinq").join("config.toml"))
    );
    // Windows takes a drive for the root of an absolute path
    let show = Path::new(if cfg!(windows) { r"C:\" } else { "/" }).join("show");
    assert_eq!(config::file(Some(&show)), Some(show.join("config.toml")));
    assert!(config::file(Some(Path::new("show"))).is_some_and(|file| file.is_absolute()));
}

#[test]
fn file_sets_the_seconds_ahead_to_make_sounds_ready() {
    let (_, sound, _, problems) = config::parse("");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(sound.ready_ahead, READY_AHEAD);
    let message = "ready_ahead should be a number of seconds, 0 or more";
    let cases = [
        ("10", 10.0, None),
        ("2.5", 2.5, None),
        ("0", 0.0, None),
        ("-1", READY_AHEAD, Some(message)),
        ("\"x\"", READY_AHEAD, Some(message)),
    ];
    cases.into_iter().for_each(|(value, seconds, problem)| {
        let (_, sound, _, problems) = config::parse(&format!("ready_ahead = {value}\n"));
        assert_eq!(sound.ready_ahead, seconds, "{value}");
        assert_eq!(messages(&problems), problem.as_slice(), "{value}");
    });
}

#[test]
fn file_names_the_audio_device_that_sound_plays_on() {
    let (_, sound, _, problems) = config::parse("");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(sound.device, None);
    let message = "audio_device should be an ID that cinq --audio-devices lists";
    let cases = [
        (
            "\"alsa:plughw:CARD=1,DEV=0\"",
            Some("alsa:plughw:CARD=1,DEV=0"),
            None,
        ),
        // Unknown ID, which only opening the device finds
        ("\"nothing\"", Some("nothing"), None),
        ("\"\"", None, Some(message)),
        ("3", None, Some(message)),
    ];
    cases.into_iter().for_each(|(value, device, problem)| {
        let (_, sound, _, problems) = config::parse(&format!("audio_device = {value}\n"));
        assert_eq!(sound.device.as_deref(), device, "{value}");
        assert_eq!(messages(&problems), problem.as_slice(), "{value}");
    });
}

#[test]
fn file_made_for_editing_changes_no_key_and_leaves_a_file_there_alone() -> io::Result<()> {
    let home = TempDir::new()?;
    let file = home.path().join("cinq").join("config.toml");
    config::make(&file)?;
    let made = config::load(Some(&file));
    assert!(matches!(&made.source, config::Source::File(read) if *read == file));
    assert!(made.problems.is_empty(), "{:?}", made.problems);
    assert_eq!(made.keymap.name(), "vi");
    fs::write(&file, "keymap = \"booth\"\n")?;
    config::make(&file)?;
    assert_eq!(fs::read_to_string(&file)?, "keymap = \"booth\"\n");
    Ok(())
}

#[test]
fn file_gives_the_address_to_take_osc_commands_on() {
    let (_, _, listen, problems) = config::parse("");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(listen, None);
    let message = "osc_listen must be IP:port, as 127.0.0.1:53000";
    let host = "osc_listen must be IP:port, not a host name";
    let space = "osc_listen must be IP:port, with no space around it";
    let cases = [
        ("\" 127.0.0.1:53000\"", None, Some(space)),
        ("\"127.0.0.1:53000 \"", None, Some(space)),
        ("\" localhost:53000\"", None, Some(host)),
        ("\" 0.0.0.0:0\"", None, Some(message)),
        ("\"0.0.0.0:53000\"", Some("0.0.0.0:53000"), None),
        ("\"127.0.0.1:53000\"", Some("127.0.0.1:53000"), None),
        ("\"[::]:53000\"", Some("[::]:53000"), None),
        ("\"localhost:53000\"", None, Some(host)),
        ("\"0.0.0.0\"", None, Some(message)),
        ("\"0.0.0.0:0\"", None, Some(message)),
        ("\"0.0.0.0:99999\"", None, Some(message)),
        ("53000", None, Some(message)),
    ];
    cases.into_iter().for_each(|(value, address, problem)| {
        let (_, _, listen, problems) = config::parse(&format!("osc_listen = {value}\n"));
        assert_eq!(
            listen,
            address.map(|address| address.parse().unwrap()),
            "{value}"
        );
        assert_eq!(messages(&problems), problem.as_slice(), "{value}");
    });
}
