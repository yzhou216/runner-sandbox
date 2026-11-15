// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sounds of the runs that the workspace has, each on the devices of the
//! outputs that its cue sends to
//!
//! A run whose sends change while it plays ramps to its new gains.  One
//! that comes to play on other devices starts again on them from where it
//! is, as a sound cannot move from one device to another.

use std::{collections::HashMap, path::PathBuf};

use itertools::Itertools;

use super::{
    Audio,
    mixer::{Gains, Mixer},
    patch::Key,
    preload::{Preloader, Wanted},
    voice::{Mixers, Target, Voice},
};
use crate::back::{Active, Clip, Command, Cue, CueId, Problem, Workspace, load::unplayable};

impl Audio {
    pub(super) fn play(&mut self, workspace: &Workspace) -> Vec<Command> {
        let Self {
            patch,
            mixers,
            voices,
            preloader,
            ..
        } = self;
        let sounding: HashMap<CueId, (&Cue, &Active, Clip)> = workspace
            .active()
            .iter()
            .filter_map(|active| {
                let cue = workspace.cue(active.id)?;
                Some((cue.id, (cue, active, active.sound()?)))
            })
            .collect();
        let channels =
            |mixers: &Mixers, key: &Key| mixers.get(key)?.as_ref().ok().map(Mixer::channels);
        let gains: HashMap<CueId, Vec<(Key, Gains)>> = sounding
            .iter()
            .map(|(id, (cue, ..))| {
                let gains = patch.gains(&workspace.sends(cue), |key| channels(mixers, key));
                (*id, gains)
            })
            .collect();
        // Voices that play on the devices they did, and those whose cue has
        // stopped, which GO has fired again, or which plays elsewhere now
        let (mut kept, ended): (HashMap<_, _>, HashMap<_, _>) =
            std::mem::take(voices).into_iter().partition(|(id, voice)| {
                let same = |gains: &Vec<(Key, Gains)>| {
                    voice
                        .keys()
                        .into_iter()
                        .eq(gains.iter().map(|(key, _)| key))
                };
                sounding
                    .get(id)
                    .is_some_and(|(_, active, _)| active.run == voice.run)
                    && gains.get(id).is_some_and(same)
            });
        ended.into_values().for_each(|voice| voice.stop(mixers));
        let (playing, failures): (_, Vec<Command>) = sounding
            .into_iter()
            .map(|(id, (cue, active, clip))| {
                let wanted = gains.get(&id).cloned().unwrap_or_default();
                let voice = match kept.remove(&id) {
                    Some(voice) => Ok(voice.routed(&wanted, mixers)),
                    None => started(mixers, wanted, preloader, workspace.path(cue), active, clip),
                };
                voice
                    .map(|voice| (id, voice.followed(active)))
                    .map_err(|reason| unplayable(cue, reason))
            })
            .partition_result();
        *voices = playing;
        let plays =
            |key: &Key| matches!(mixers.get(key), Some(Ok(mixer)) if mixer.failure().is_none());
        let heard = voices
            .iter()
            .filter_map(|(&id, voice)| voice.heard(workspace.active_of(id)?, plays));
        failures.into_iter().chain(heard).collect()
    }
}

/// Starts the sound of a run on each device that `gains` name, from the
/// sound made ready for it where there is one
fn started(
    mixers: &mut Mixers,
    gains: Vec<(Key, Gains)>,
    preloader: &mut Preloader,
    path: Option<PathBuf>,
    active: &Active,
    clip: Clip,
) -> Result<Voice, String> {
    let path = path.ok_or_else(|| Problem::NoFile.to_string())?;
    let wanted = Wanted { path, clip };
    let targets = targets(mixers, gains);
    match preloader.take(active.id, &wanted) {
        Some(Ok(sound)) => Voice::start(targets, sound, &wanted.path, active, clip),
        // Sound that could not be made ready, whose file may have come back
        // since
        _ => Voice::open(targets, &wanted.path, active, clip),
    }
}

/// Returns the mixer of each device that `gains` name, in their order,
/// leaving out those that could not open
fn targets(mixers: &mut Mixers, gains: Vec<(Key, Gains)>) -> Vec<Target<'_>> {
    mixers
        .iter_mut()
        .filter_map(|(key, mixer)| {
            let order = gains.iter().position(|(wanted, _)| wanted == key)?;
            let target = Target {
                key: key.clone(),
                mixer: mixer.as_mut().ok()?,
                gains: gains[order].1.clone(),
            };
            Some((order, target))
        })
        .sorted_by_key(|(order, _)| *order)
        .map(|(_, target)| target)
        .collect()
}
