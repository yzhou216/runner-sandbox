// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the audio device that cinq keeps in the text of the file

use std::{fs, io};

use tempfile::TempDir;

use super::{keep_audio_device, with_audio_device};

const MADE: &str = "# Settings of cinq\n";

#[test]
fn new_setting_goes_after_the_comments_and_before_the_tables() {
    let device = Some("alsa:plughw:CARD=1,DEV=0");
    let cases = [
        ("", "audio_device = \"alsa:plughw:CARD=1,DEV=0\"\n"),
        (
            MADE,
            "# Settings of cinq\naudio_device = \"alsa:plughw:CARD=1,DEV=0\"\n",
        ),
        (
            "# Settings of cinq\n\nkeymap = \"cua\" # Booth keys\n[keys]\ngo = \"F5\"\n",
            "# Settings of cinq\n\naudio_device = \"alsa:plughw:CARD=1,DEV=0\"\nkeymap = \"cua\" # Booth keys\n[keys]\ngo = \"F5\"\n",
        ),
        (
            "# No line break at the end",
            "# No line break at the end\naudio_device = \"alsa:plughw:CARD=1,DEV=0\"\n",
        ),
    ];
    cases.into_iter().for_each(|(text, expected)| {
        assert_eq!(
            with_audio_device(text, device).as_deref(),
            Ok(expected),
            "{text:?}"
        );
    });
}

#[test]
fn setting_there_changes_in_place_or_goes_for_the_default_device() {
    let text = "keymap = \"cua\"\naudio_device = \"alsa:default\" # House\nready_ahead = 5\n";
    assert_eq!(
        with_audio_device(text, Some("alsa:pipewire")).as_deref(),
        Ok("keymap = \"cua\"\naudio_device = \"alsa:pipewire\" # House\nready_ahead = 5\n")
    );
    assert_eq!(
        with_audio_device(text, None).as_deref(),
        Ok("keymap = \"cua\"\nready_ahead = 5\n")
    );
    assert_eq!(with_audio_device(MADE, None).as_deref(), Ok(MADE));
}

#[test]
fn id_with_quotes_stays_one_string() {
    let id = "coreaudio:Studio \"A\" \\ B";
    let text = with_audio_device("", Some(id)).expect("empty text is TOML");
    let read: toml::Table = toml::from_str(&text).expect("the setting is TOML");
    assert_eq!(read["audio_device"].as_str(), Some(id));
}

#[test]
fn file_that_is_not_toml_keeps_no_device() -> io::Result<()> {
    let home = TempDir::new()?;
    let file = home.path().join("config.toml");
    fs::write(&file, "[keys\n")?;
    let kept = keep_audio_device(&file, Some("alsa:default"));
    assert_eq!(
        kept,
        Err("config.toml cannot keep the audio device: it is not TOML".to_owned())
    );
    assert_eq!(fs::read_to_string(&file)?, "[keys\n");
    Ok(())
}

#[test]
fn missing_file_is_made_to_keep_the_device() -> io::Result<()> {
    let home = TempDir::new()?;
    let file = home.path().join("cinq").join("config.toml");
    assert_eq!(keep_audio_device(&file, Some("alsa:default")), Ok(()));
    assert!(fs::read_to_string(&file)?.ends_with("\naudio_device = \"alsa:default\"\n"));
    Ok(())
}
