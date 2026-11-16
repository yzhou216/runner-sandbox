// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the sound of a run plays: a manager of its own on a mixer
//!
//! Each sound plays in a kira manager whose renderer the mixer of its
//! device takes, as [`crate::back::audio::feed`] tells.

use std::path::Path;

use kira::{
    AudioManager, AudioManagerSettings, Capacities,
    sound::{
        FromFileError, Region, SoundData,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
    track::MainTrackBuilder,
};

use crate::back::{
    Active, Clip,
    audio::{
        feed::Feed,
        mixer::{Gains, Mixer},
        patch::Key,
    },
};

pub(super) struct Play {
    pub(super) sound: StreamingSoundHandle<FromFileError>,
    /// Place of the sound in the mixer of its device, which a test that
    /// plays the sound in a manager of its own has none of
    pub(super) place: Option<Place>,
}

pub(super) struct Place {
    pub(in crate::back::audio) key: Key,
    /// Identity of the sound in the mixer
    pub(super) id: u64,
    pub(super) gains: Gains,
    /// Manager of the sound alone, which hands the sound its commands
    pub(super) _manager: AudioManager<Feed>,
}

/// Mixer that a sound of a run starts in, with the gains it plays at
pub(in crate::back::audio) struct Target<'a> {
    pub(in crate::back::audio) key: Key,
    pub(in crate::back::audio) mixer: &'a mut Mixer,
    pub(in crate::back::audio) gains: Gains,
}

/// Plays on each of `targets` in turn as `play` does, and returns the
/// sounds.  Where one fails, the sounds that started on the targets before
/// it stop, since nothing else would own them, and its reason is returned.
pub(super) fn each(
    targets: &mut [Target<'_>],
    mut play: impl FnMut(&mut Target<'_>) -> Result<Play, String>,
) -> Result<Vec<Play>, String> {
    let mut plays = Vec::with_capacity(targets.len());
    let failure = targets
        .iter_mut()
        .find_map(|target| play(target).map(|played| plays.push(played)).err());
    match failure {
        None => Ok(plays),
        Some(reason) => {
            plays
                .into_iter()
                .filter_map(|play| play.place)
                .for_each(|place| {
                    targets
                        .iter_mut()
                        .filter(|target| target.key == place.key)
                        .for_each(|target| target.mixer.remove(place.id));
                });
            Err(reason)
        }
    }
}

pub(super) fn opened(
    target: &mut Target<'_>,
    file: &Path,
    active: &Active,
    clip: Clip,
) -> Result<Play, String> {
    let data = StreamingSoundData::from_file(file).map_err(crate::back::audio::reason)?;
    let data = clipped(data, clip, clip.position(active.played()), active.level);
    placed(target, data)
}

/// Plays `data` in a manager of its own, whose renderer the mixer of
/// `target` takes and mixes at the gains of `target`
pub(super) fn placed<D>(target: &mut Target<'_>, data: D) -> Result<Play, String>
where
    D: SoundData<Handle = StreamingSoundHandle<FromFileError>>,
    D::Error: std::fmt::Display,
{
    let Ok(mut manager) = AudioManager::<Feed>::new(AudioManagerSettings {
        capacities: Capacities {
            sub_track_capacity: 1,
            send_track_capacity: 1,
            clock_capacity: 1,
            modulator_capacity: 1,
            listener_capacity: 1,
        },
        main_track_builder: MainTrackBuilder::new().sound_capacity(1),
        backend_settings: target.mixer.sample_rate(),
        ..AudioManagerSettings::default()
    });
    let sound = manager.play(data).map_err(|error| error.to_string())?;
    let renderer = manager
        .backend_mut()
        .take()
        .expect("kira hands its renderer to the backend as the manager opens");
    let id = target.mixer.add(renderer, target.gains.clone())?;
    Ok(Play {
        sound,
        place: Some(Place {
            key: target.key.clone(),
            id,
            gains: target.gains.clone(),
            _manager: manager,
        }),
    })
}

/// Returns `data` set to play `clip`, from `position` in the file and at
/// `level`.
pub(in crate::back::audio) fn clipped(
    data: StreamingSoundData<FromFileError>,
    clip: Clip,
    position: f64,
    level: f32,
) -> StreamingSoundData<FromFileError> {
    // Cut from the start of the file, so that times in the cut are times in
    // the file, which kira seeks to directly.  A cut is left out where it
    // would end past the last frame of the file.
    let cut = (clip.end < data.duration().as_secs_f64()).then(|| Region::from(0.0..clip.end));
    let looped = clip.looping.then(|| Region::from(clip.start..));
    data.slice(cut)
        .loop_region(looped)
        .start_position(position)
        .volume(level)
}
