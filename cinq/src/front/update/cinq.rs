// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Interface together with the parts that have side effects: the sound,
//! video, light, and network output, the keeper of unsaved changes, and the
//! windows

use std::path::{Path, PathBuf};

use iced::{Element, Subscription, Task, window};

use super::{Keeper, device, show};
use crate::{
    back::{Audio, Lights, Network, Video},
    front::{App, Cinq, MAIN_SIZE, Message, keymap, view, window_settings},
};

impl Cinq {
    /// Starts opening the show in `show`, or a new show without one, which
    /// the window opens without waiting for
    pub(in crate::front) fn new(
        config: Option<&Path>,
        show: Option<PathBuf>,
    ) -> (Self, Task<Message>) {
        let config = keymap::config::load(config);
        let audio = Audio::open(config.sound.device.as_deref());
        let keeper = Keeper::new(show::unsaved_directory());
        let reader = keeper.reader();
        let (main, opened) = window::open(window_settings(MAIN_SIZE));
        let cinq = Self {
            app: App {
                main: Some(main),
                audio_error: audio.error().map(str::to_owned),
                config,
                reader: reader.clone(),
                ..App::default()
            },
            audio,
            video: Video::default(),
            lights: Lights::open(),
            network: Network::open(),
            keeper,
        };
        let opening = show::open(show, reader);
        (
            cinq,
            Task::batch([opened.discard(), opening, device::listing()]),
        )
    }

    /// Applies one message and lets the parts with side effects catch up.
    /// This is the only place where state is replaced, and the video,
    /// sound, light, and network output and the keeper of unsaved changes
    /// are the only parts it calls that have side effects; everything else
    /// builds new values.  The video output goes first, so that a run whose
    /// picture cannot play has its sound stopped in the same update.  Each
    /// reading of the configuration file and each choice of an audio device
    /// routes the sound to the device chosen, or tries again to open one
    /// that failed.  Taking the state out of `self` instead of cloning it
    /// keeps updates free of copies.
    pub(in crate::front) fn update(&mut self, message: Message) -> Task<Message> {
        let routed = matches!(
            message,
            Message::Reconfigured(_) | Message::ChooseAudioDevice(_)
        );
        let (app, task) = std::mem::take(&mut self.app).next(message);
        if routed {
            self.audio.route(app.config.sound.device.as_deref());
        }
        let (workspace, pictures) = self.video.sync(app.workspace);
        let due = self.video.next_due(&workspace);
        let workspace = self.audio.sync(workspace, app.config.sound.ready_ahead);
        self.lights.sync(&workspace);
        let (workspace, messages) = workspace.drained();
        self.network.send(messages);
        let kept = self.keeper.keep(&app.show);
        let app = App {
            stage: app.stage.with_pictures(&workspace, pictures).with_due(due),
            audio_error: self.audio.error().map(str::to_owned),
            network_problems: self.network.problems(),
            network_checks: self.network.checks(),
            workspace,
            ..app
        }
        .with_keep_problem(kept)
        .clocked();
        let (app, load) = app.load();
        let (app, stage) = app.staged();
        self.app = app;
        Task::batch([task, load, stage])
    }

    pub(in crate::front) fn view(&self, window: window::Id) -> Element<'_, Message> {
        if self.app.stage.is_window(window) {
            view::stage::view(&self.app)
        } else {
            view::view(&self.app)
        }
    }

    pub(in crate::front) fn title(&self, window: window::Id) -> String {
        let show = &self.app.show;
        if self.app.stage.is_window(window) {
            format!("{} stage - cinq", show.name())
        } else {
            format!("{}{} - cinq", show.name(), show.mark())
        }
    }

    pub(in crate::front) fn subscription(&self) -> Subscription<Message> {
        self.app.subscription()
    }
}
