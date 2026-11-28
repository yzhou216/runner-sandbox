// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Interface together with the parts that have side effects: the sound,
//! video, light, and network output, the keeper of unsaved changes, and the
//! windows.  This is the only place where state is replaced.

use std::path::{Path, PathBuf};

use iced::{Element, Subscription, Task, window};

use super::{Keeper, device, dispatch::passive, focus::held, hints::Above, show};
use crate::{
    back::{Audio, Lights, Network, Video},
    front::{App, Cinq, MAIN_SIZE, Message, Problems, config, view, window_settings},
};

impl Cinq {
    /// Starts opening the show in `show`, or a new show without one, which
    /// the window opens without waiting for
    pub(in crate::front) fn new(
        config: Option<&Path>,
        show: Option<PathBuf>,
    ) -> (Self, Task<Message>) {
        let config = config::load(config);
        let audio = Audio::open(config.sound.device.as_deref());
        let (lights, light_reports) = Lights::open();
        let keeper = Keeper::new(show::unsaved_directory());
        let reader = keeper.reader();
        let (main, opened) = window::open(window_settings(MAIN_SIZE));
        let cinq = Self {
            app: App {
                main: Some(main),
                problems: Problems {
                    audio: audio.problems(),
                    ..Problems::default()
                },
                config,
                reader: reader.clone(),
                ..App::default()
            },
            audio,
            video: Video::default(),
            lights,
            light_reports,
            network: Network::open(),
            keeper,
        };
        let opening = show::open(show, reader);
        (
            cinq,
            Task::batch([opened.discard(), opening, device::listing()]),
        )
    }

    /// Applies one message and lets the parts with side effects catch up,
    /// as [`Cinq::synced`] tells.  Taking the state out of `self` instead
    /// of cloning it keeps updates free of copies.
    pub(in crate::front) fn update(&mut self, message: Message) -> Task<Message> {
        let routed = matches!(
            message,
            Message::Reconfigured(_) | Message::ChooseAudioDevice(_)
        );
        // Whatever the operator does can move the keyboard between fields,
        // which only the widgets know
        let asking = !passive(&message);
        if let Message::LightTold(problem) = &message {
            self.lights.told(problem.clone());
        }
        let above = self.app.hints.is_some().then(|| self.app.above_places());
        let (app, task) = std::mem::take(&mut self.app).next(message);
        let (app, load) = self.synced(app, routed, above.as_ref()).load();
        let (app, stage) = app.staged();
        self.app = app;
        let task = if asking { task.chain(held()) } else { task };
        Task::batch([task, load, stage])
    }

    /// Returns `app` after the outputs have caught up with it.  The video
    /// output goes first, so that a run whose picture cannot play has its
    /// sound stopped in the same update.  A `routed` message, a reading of
    /// the configuration file or a choice of an audio device, routes the
    /// sound to the device chosen.  Hint mode ends unless the sidebar still
    /// holds what it held `above`.
    fn synced(&mut self, app: App, routed: bool, above: Option<&Above>) -> App {
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
        App {
            stage: app.stage.with_pictures(&workspace, pictures).with_due(due),
            // Checks first: once they stop, the problems read after them are
            // final
            network_checks: self.network.checks(),
            problems: Problems {
                audio: self.audio.problems(),
                light: self.lights.problem().map(str::to_owned),
                network: self.network.problems(&workspace.destinations()),
                network_failed: self.network.failed(&workspace.destinations()),
                ..app.problems
            },
            workspace,
            ..app
        }
        .with_keep_problem(kept)
        .hints_after(above)
        .clocked()
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
        Subscription::batch([
            self.app.subscription(),
            self.light_reports.subscription().map(Message::LightTold),
        ])
    }
}
