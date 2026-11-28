// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! iced interface for a workspace
//!
//! The interface keeps only its own state, such as the selection and the
//! open inspector tab.  It reads the show from the back end and changes it
//! only by sending back-end commands.  Operator input arrives as a
//! [`Message`], [`update`] applies it, and [`view`] draws the result.  Keys
//! become messages through [`keymap`], whose bindings the [`config`] file
//! can change.  A stage window shows the pictures of picture and video
//! cues once the show holds one.

mod app;
mod config;
mod files;
#[cfg(test)]
mod fixtures;
mod keymap;
mod message;
mod stepping;
mod update;
mod view;

use std::{
    env,
    path::{self, PathBuf},
};

use iced::{Size, window};

use crate::back::{Audio, LightReports, Lights, Network, Video};
use app::{App, Notices, Problems};
use message::{Caret, Focus, Message, Setting, Spot, Tab};
use update::{History, Keeper, Search};

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
    let config = config::file(config_dir.as_deref());
    let show = show.map(|show| path::absolute(&show).unwrap_or(show));
    config::guard();
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
    config::restore();
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

/// Returns the error that stands in for a winit panic when no variable names
/// a Wayland compositor, on the platforms where cinq opens its window on
/// Wayland alone
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

/// Interface state with the parts that have side effects, which only
/// [`Cinq::update`] reaches
struct Cinq {
    app: App,
    audio: Audio,
    video: Video,
    lights: Lights,
    light_reports: LightReports,
    network: Network,
    keeper: Keeper,
}
