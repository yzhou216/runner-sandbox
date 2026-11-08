// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of watching the configuration file, whose keys follow the file as
//! it comes, as an editor saves it, and as it goes, whatever becomes of the
//! problems written out

use std::{
    fs,
    io::{self, Write},
    iter,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Duration,
};

use futures::executor;
use iced::futures::{StreamExt, future};
use tempfile::TempDir;

use super::{Errors, watch};
use crate::front::keymap::{
    Preset,
    config::{Config, Source},
};

/// Time that the watcher has to see a change before the test fails
const PATIENCE: Duration = Duration::from_secs(10);

/// Waits for a configuration from the watcher that `wanted` accepts
fn until(configs: &mpsc::Receiver<Config>, wanted: impl Fn(&Config) -> bool) {
    let found = iter::from_fn(|| configs.recv_timeout(PATIENCE).ok()).any(|config| wanted(&config));
    assert!(found, "the watcher sent no such configuration");
}

/// Accepts a configuration read from a file that starts from `preset`
fn from_file(preset: Preset) -> impl Fn(&Config) -> bool {
    move |config| matches!(config.source, Source::File(_)) && config.keymap.name() == preset.name()
}

/// Accepts the defaults of a configuration whose file is missing
fn defaults(config: &Config) -> bool {
    matches!(config.source, Source::Missing(_))
}

/// Watches `file` on a thread of its own, writing problems to `errors`, and
/// returns what the watcher sends
fn watched(file: PathBuf, errors: Errors) -> mpsc::Receiver<Config> {
    let (sender, configs) = mpsc::channel();
    thread::spawn(move || {
        executor::block_on(watch(file, errors).for_each(|config| {
            let _ = sender.send(config);
            future::ready(())
        }));
    });
    configs
}

/// Standard error that takes nothing written to it, as a terminal paused
/// with Control+s does
struct Paused;

impl Write for Paused {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        loop {
            thread::park();
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn keys_follow_the_file_as_it_comes_changes_and_goes() -> io::Result<()> {
    let home = TempDir::new()?;
    // Directories of the file, which come later as well
    let directory = home.path().join("config").join("cinq");
    let file = directory.join("config.toml");
    let configs = watched(file.clone(), Errors::Log(Box::new(io::sink())));
    until(&configs, defaults);
    fs::create_dir_all(&directory)?;
    fs::write(&file, "keymap = \"cua\"\n")?;
    until(&configs, from_file(Preset::Cua));
    // Saved as editors often save, by renaming another file over it
    let draft = directory.join("config.toml.draft");
    fs::write(&draft, "keymap = \"vi\"\n")?;
    fs::rename(&draft, &file)?;
    until(&configs, from_file(Preset::Vi));
    fs::remove_file(&file)?;
    until(&configs, defaults);
    Ok(())
}

#[test]
fn reading_the_file_does_not_make_the_watcher_read_it_again() -> io::Result<()> {
    let home = TempDir::new()?;
    let file = home.path().join("config.toml");
    fs::write(&file, "keymap = \"cua\"\n")?;
    let configs = watched(file, Errors::Log(Box::new(io::sink())));
    until(&configs, from_file(Preset::Cua));
    // Reading opens the file, which the watcher sees as well, so a watcher
    // that took the opening for a change would read the file over and over.
    // FSEvents on Darwin may still report the write made just before the
    // watch began, which reads the file once more, and no more.
    let again = iter::from_fn(|| configs.recv_timeout(Duration::from_secs(2)).ok())
        .take(2)
        .count();
    assert!(again <= 1, "the watcher kept reading the file unasked");
    Ok(())
}

#[cfg(unix)]
#[test]
fn changes_by_the_real_path_of_a_linked_directory_touch_the_file() -> io::Result<()> {
    use std::{os::unix::fs::symlink, time::Instant};

    use notify_debouncer_full::{
        DebouncedEvent,
        notify::{Event, EventKind, event::CreateKind},
    };

    use super::{names, touches};

    let home = TempDir::new()?;
    let real = home.path().join("real");
    fs::create_dir(&real)?;
    let link = home.path().join("link");
    symlink(&real, &link)?;
    let file = link.join("config.toml");
    // FSEvents on Darwin reports a change past every symbolic link
    let event = Event::new(EventKind::Create(CreateKind::File))
        .add_path(real.canonicalize()?.join("config.toml"));
    let burst = Ok(vec![DebouncedEvent::new(event, Instant::now())]);
    assert!(touches(&burst, &names(&file, Some(&link))));
    Ok(())
}

#[test]
fn a_save_comes_back_formatted() -> io::Result<()> {
    let home = TempDir::new()?;
    let file = home.path().join("config.toml");
    fs::write(&file, "keymap = \"cua\"\n")?;
    let configs = watched(file.clone(), Errors::Log(Box::new(io::sink())));
    until(&configs, from_file(Preset::Cua));
    fs::write(&file, "keymap=\"vi\"\n")?;
    // Reading that brings the keys of the save formats the file first
    until(&configs, from_file(Preset::Vi));
    assert_eq!(fs::read_to_string(&file)?, "keymap = \"vi\"\n");
    Ok(())
}

#[test]
fn keys_follow_the_file_while_standard_error_takes_nothing() -> io::Result<()> {
    let home = TempDir::new()?;
    let file = home.path().join("config.toml");
    fs::write(&file, "colour = 1\n")?;
    let configs = watched(file.clone(), Errors::Log(Box::new(Paused)));
    // Problems of the first reading leave the thread that writes them
    // waiting for good
    until(&configs, |config| !config.problems.is_empty());
    fs::write(&file, "keymap = \"cua\"\n")?;
    until(&configs, from_file(Preset::Cua));
    Ok(())
}
