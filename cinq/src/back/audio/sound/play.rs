// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sounds of the runs that the workspace has, each on the devices of the
//! outputs that its cue sends to
//!
//! A run whose sends change while it plays ramps to its new gains.  One
//! that comes to play on other devices starts again on them only, from
//! where it is, as a sound cannot move from one device to another, while
//! its sounds on the devices it keeps play on unbroken.

use std::{collections::HashMap, path::PathBuf};

use itertools::Itertools;

use crate::{
    back::{
        Active, Clip, Command, Cue, CueId, Problem, Workspace,
        audio::{
            Audio,
            patch::{Gains, Key, Patch},
            sound::{
                mixer::{Mixers, channels_of},
                preload::{Preloader, Wanted},
                voice::{Target, Voice},
            },
        },
        failure::unplayable,
    },
    warn,
};

/// Cues whose sound plays, each with its active run and its clip
type Sounding<'a> = HashMap<CueId, (&'a Cue, &'a Active, Clip)>;

/// Gains of each cue on the devices that its sends name
type Routes = HashMap<CueId, Vec<(Key, Gains)>>;

impl Audio {
    pub(in crate::back::audio) fn play(&mut self, workspace: &Workspace) -> Vec<Command> {
        let Self {
            patch,
            mixers,
            voices,
            begun,
            preloader,
            ..
        } = self;
        let sounding = sounding(workspace);
        let mut gains = routes(patch, mixers, workspace, &sounding);
        let (mut kept, ended) = settled(std::mem::take(voices), &sounding);
        ended.into_values().for_each(|voice| voice.stop(mixers));
        let (playing, failures): (HashMap<_, _>, Vec<Command>) = sounding
            .into_iter()
            .map(|(id, (cue, active, clip))| {
                let wanted = gains.remove(&id).unwrap_or_default();
                let voice = if let Some(voice) = kept.remove(&id) {
                    reconciled(mixers, voice, wanted, workspace.path(cue), active, clip)
                } else {
                    let again = begun.get(&id) == Some(&active.run);
                    started(
                        mixers,
                        &wanted,
                        (!again).then_some(&mut *preloader),
                        workspace.path(cue),
                        active,
                        clip,
                    )
                };
                voice
                    .map(|voice| (id, voice.followed(active)))
                    .map_err(|reason| unplayable(workspace, cue, reason))
            })
            .partition_result();
        *begun = playing.iter().map(|(&id, voice)| (id, voice.run)).collect();
        *voices = playing;
        let reports = reports(workspace, voices, mixers);
        failures.into_iter().chain(reports).collect()
    }
}

fn sounding(workspace: &Workspace) -> Sounding<'_> {
    workspace
        .active()
        .iter()
        .filter_map(|active| {
            let cue = workspace.cue(active.id)?;
            Some((cue.id, (cue, active, active.sound()?)))
        })
        .collect()
}

fn routes(patch: &Patch, mixers: &Mixers, workspace: &Workspace, sounding: &Sounding) -> Routes {
    sounding
        .iter()
        .map(|(id, (cue, ..))| {
            let gains = patch.gains(&workspace.sends(cue), |key| channels_of(mixers, key));
            (*id, gains)
        })
        .collect()
}

/// Returns the voices whose run goes on, and those whose cue has stopped or
/// which GO has fired again
fn settled(
    voices: HashMap<CueId, Voice>,
    sounding: &Sounding,
) -> (HashMap<CueId, Voice>, HashMap<CueId, Voice>) {
    voices.into_iter().partition(|(id, voice)| {
        sounding
            .get(id)
            .is_some_and(|(_, active, _)| active.run == voice.run)
    })
}

/// Returns the voice of a run that goes on, as it plays on the devices that
/// `gains` name now.  Moving an output among the devices that it plays on
/// leaves the sound where it is.
fn reconciled(
    mixers: &mut Mixers,
    voice: Voice,
    gains: Vec<(Key, Gains)>,
    path: Option<PathBuf>,
    active: &Active,
    clip: Clip,
) -> Result<Voice, String> {
    let voice = voice
        .retained(|key| gains.iter().any(|(wanted, _)| wanted == key), mixers)
        .routed(&gains, mixers);
    let missing: Vec<_> = gains
        .into_iter()
        .filter(|(key, _)| !voice.keys().contains(&key))
        .collect();
    if missing.is_empty() {
        return Ok(voice);
    }
    let Some(path) = path else {
        voice.stop(mixers);
        return Err(Problem::NoFile.to_string());
    };
    voice
        .joined(targets(mixers, &missing), &path, active, clip)
        .map_err(|(reason, voice)| {
            voice.stop(mixers);
            reason
        })
}

/// Returns the commands that tell of the voices that were heard, and of those
/// whose sound stopped before its end
fn reports(workspace: &Workspace, voices: &HashMap<CueId, Voice>, mixers: &Mixers) -> Vec<Command> {
    let plays = |key: &Key| matches!(mixers.get(key), Some(Ok(mixer)) if mixer.failure().is_none());
    let heard = voices
        .iter()
        .filter_map(|(&id, voice)| voice.heard(workspace.active_of(id)?, plays));
    let cut = voices.iter().filter_map(|(&id, voice)| {
        let cue = workspace.cue(id)?;
        let clip = workspace.active_of(id)?.sound()?;
        voice
            .cut_short(clip, cue.sound_ends(clip)?, plays)
            .then(|| ended_early(workspace, cue))
    });
    heard.chain(cut).collect()
}

/// Returns the command that records that the sound of `cue` stopped before
/// its end, as it does where its file is cut short or holds more than two
/// channels, which the decoder refuses, and tells so on standard error, once
/// for the file.  Playing on by the clock of the show, the cue would stay
/// green and silent.
fn ended_early(workspace: &Workspace, cue: &Cue) -> Command {
    let reason = "its sound stopped early".to_owned();
    workspace
        .path(cue)
        .iter()
        .for_each(|path| warn::once(&format!("{}: {reason}", path.display())));
    unplayable(workspace, cue, reason)
}

/// Starts the sound of a run on each device that `gains` name, from the
/// sound that `preloader` made ready for it where there is one.  A run that
/// starts again, on other devices, has none to take, since the sound made
/// ready waits at the start of its clip and is for the next GO of its cue.
fn started(
    mixers: &mut Mixers,
    gains: &[(Key, Gains)],
    preloader: Option<&mut Preloader>,
    path: Option<PathBuf>,
    active: &Active,
    clip: Clip,
) -> Result<Voice, String> {
    let path = path.ok_or_else(|| Problem::NoFile.to_string())?;
    let wanted = Wanted { path, clip };
    let targets = targets(mixers, gains);
    // With no device to play on, the sound made ready stays for the device
    // that comes to play the run
    let ready = preloader
        .filter(|_| !targets.is_empty())
        .and_then(|preloader| preloader.take(active.id, &wanted));
    match ready {
        Some(Ok(sound)) => Voice::start(targets, sound, &wanted.path, active, clip),
        // Sound that could not be made ready, whose file may have come back
        // since
        _ => Voice::open(targets, &wanted.path, active, clip),
    }
}

/// Returns the mixer of each device that `gains` name, in their order,
/// leaving out those that could not open
fn targets<'a>(mixers: &'a mut Mixers, gains: &[(Key, Gains)]) -> Vec<Target<'a>> {
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
