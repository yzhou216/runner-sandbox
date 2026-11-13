// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! iced interface for a workspace
//!
//! The interface keeps only its own state, such as the selection and the
//! open inspector tab.  It reads the show from the back end and changes it
//! only by sending back-end commands.  Operator input arrives as a
//! [`Message`], [`update`] applies it, and [`view`] draws the result.  Keys
//! become messages through [`keymap`], whose bindings a configuration file
//! can change.  Besides the main window, a stage window shows the pictures
//! of picture and video cues once the show holds one.

mod keymap;
mod message;
mod update;
mod view;

use std::{
    env, fs, io,
    path::{self, Path, PathBuf},
    time::Instant,
};

use iced::{Size, window};

use crate::back::{Audio, CueId, CueList, Kind, Lights, Video, Workspace};
use keymap::{Press, config::Config};
use message::{Entry, Focus, Message, Tab};
use update::{Keeper, Reader, Show, Stage};

/// Opens the main window with the show in `show`, or a new show without
/// one, and runs the interface until the main window closes.  The key
/// bindings come from `config.toml` in `config_dir`, when it is given, and
/// otherwise from the platform's configuration directory.  A terminal that
/// shows problems with the file on a screen of their own switches back
/// however the interface ends.
pub fn run(config_dir: Option<PathBuf>, show: Option<PathBuf>) -> iced::Result {
    if let Some(missing) = no_wayland() {
        return Err(missing);
    }
    let config = keymap::config::file(config_dir.as_deref());
    let show = show.map(|show| path::absolute(&show).unwrap_or(show));
    keymap::config::guard();
    let ran = iced::daemon(
        move || Cinq::new(config.as_deref(), show.clone()),
        Cinq::update,
        Cinq::view,
    )
    .title(Cinq::title)
    .theme(view::theme::modus_vivendi())
    .settings(iced::Settings {
        default_font: view::FONT,
        default_text_size: view::TEXT_SIZE,
        ..iced::Settings::default()
    })
    .subscription(Cinq::subscription)
    .run();
    keymap::config::restore();
    ran
}

const MAIN_SIZE: Size = Size::new(1440.0, 900.0);

/// Shape of most video
const STAGE_SIZE: Size = Size::new(960.0, 540.0);

fn window_settings(size: Size) -> window::Settings {
    window::Settings {
        size,
        // Application ID for window manager rules, since some window
        // managers float a window whose ID iced has left empty
        #[cfg(target_os = "linux")]
        platform_specific: window::settings::PlatformSpecific {
            application_id: "cinq".to_owned(),
            ..window::settings::PlatformSpecific::default()
        },
        ..window::Settings::default()
    }
}

/// Returns why the window cannot open on GNU/Linux and the BSDs, where cinq
/// opens it on Wayland alone, when no variable names a Wayland compositor
/// to open it on, as winit tells before it tries, but only by panicking
fn no_wayland() -> Option<iced::Error> {
    let wayland_only = cfg!(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ));
    let named = ["WAYLAND_DISPLAY", "WAYLAND_SOCKET"]
        .into_iter()
        .any(|name| env::var_os(name).is_some_and(|value| !value.is_empty()));
    (wayland_only && !named).then(|| {
        iced::Error::WindowCreationFailed(
            "cinq runs on Wayland alone, and neither WAYLAND_DISPLAY nor WAYLAND_SOCKET is set"
                .into(),
        )
    })
}

/// Interface state together with the parts that have side effects, which
/// stay apart from it so that only [`Cinq::update`] reaches them
struct Cinq {
    app: App,
    audio: Audio,
    video: Video,
    lights: Lights,
    keeper: Keeper,
}

#[derive(Default)]
struct App {
    workspace: Workspace,
    /// Main window, once it has been asked for, whose closing quits cinq
    main: Option<window::Id>,
    stage: Stage,
    /// Windows that are minimized, which iced does not draw
    minimized: Vec<window::Id>,
    show: Show,
    /// What went wrong with show files last, until opening or saving one
    /// succeeds
    file_problem: Option<String>,
    /// Why the keeper failed to keep unsaved changes last, until opening or
    /// saving a show succeeds.  It arrives late and about an earlier change,
    /// so it has a slot of its own.
    keep_problem: Option<String>,
    /// What the latest opening or saving told, which the status bar shows
    /// until the show changes
    told: Option<String>,
    /// Way to read the unsaved changes kept for shows, if the platform has
    /// a directory to keep them in
    reader: Option<Reader>,
    audio_error: Option<String>,
    /// ID and name of each audio device, as they were listed last
    audio_devices: Vec<(String, String)>,
    /// Why the configuration file did not keep the audio device chosen
    /// last, until it keeps another
    device_problem: Option<String>,
    list: usize,
    /// Cue shown in the inspector
    selected: Option<CueId>,
    /// Text of a field as the operator types it, which the field shows in
    /// place of its value until the operator does anything else
    entered: Option<(Entry, String)>,
    tab: Tab,
    /// Kind of cue that Add cue creates
    new_kind: Kind,
    /// Time of the last clock tick, or `None` while the clock is stopped
    last_tick: Option<Instant>,
    config: Config,
    /// Keys of a command typed so far, such as a count or the `g` of `g g`
    keys: Vec<Press>,
    help: bool,
    /// Why the link that the help page opened last failed to open, which
    /// the page shows while it stays open
    link_problem: Option<String>,
    /// Whether the files of audio, video, and picture cues are being loaded
    loading: bool,
    /// Whether the files that could not be played are being read again
    rechecking: bool,
}

impl App {
    fn current(&self) -> &CueList {
        &self.workspace.lists()[self.list]
    }

    fn selected_here(&self) -> Option<CueId> {
        self.selected.filter(|_| self.selected_row().is_some())
    }

    fn selected_row(&self) -> Option<usize> {
        self.selected.and_then(|id| self.current().position(id))
    }

    fn editable(&self) -> bool {
        !self.workspace.show_mode()
    }

    fn running(&self) -> bool {
        self.workspace.active().iter().any(|active| !active.paused)
    }

    fn moving_pictures(&self) -> bool {
        self.workspace
            .active()
            .iter()
            .any(|active| active.shows_video() && !active.paused)
    }
}

/// Returns the item `offset` places after `from`, or before it when
/// `offset` is negative, among `count` items.  Stepping stays put past
/// either end, and without an item to step from, it starts at the first.
fn step(from: Option<usize>, offset: isize, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    Some(from.map_or(0, |index| index.saturating_add_signed(offset).min(last)))
}

/// Reads a file, or returns none when there is no such file
fn read(file: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(file) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

/// Returns where a view that shows `height` of its content from `scrolled`
/// on has to scroll to, to show the span from `top` to `bottom` while
/// scrolling as little as it can, or `None` when the span is in view
fn reveal(top: f32, bottom: f32, scrolled: f32, height: f32) -> Option<f32> {
    if top < scrolled {
        Some(top)
    } else if bottom > scrolled + height {
        Some(bottom - height)
    } else {
        None
    }
}
