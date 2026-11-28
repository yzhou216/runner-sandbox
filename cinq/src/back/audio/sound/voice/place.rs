// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the sound of a run plays: a kira manager of its own, whose renderer
//! the mixer of its device takes, as [`crate::back::audio::sound::feed`]
//! tells

use std::path::Path;

use kira::{
    AudioManager, AudioManagerSettings, Capacities,
    sound::{
        EndPosition, FromFileError, Region, SoundData,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
    track::MainTrackBuilder,
};

use crate::back::{
    Active, Clip,
    audio::{
        patch::{Gains, Key},
        sound::{feed::Feed, mixer::Mixer},
    },
};

pub(super) struct Play {
    pub(super) sound: StreamingSoundHandle<FromFileError>,
    /// Place of the sound in the mixer of its device, which a test that
    /// plays the sound in a manager of its own has none of
    pub(super) place: Option<Place>,
}

pub(super) struct Place {
    pub(super) key: Key,
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
    let capacity = targets.len();
    let started =
        targets.iter_mut().try_fold(
            Vec::with_capacity(capacity),
            |mut plays, target| match play(target) {
                Ok(played) => {
                    plays.push(played);
                    Ok(plays)
                }
                Err(reason) => Err((plays, reason)),
            },
        );
    started.map_err(|(plays, reason)| {
        stopped(targets, plays);
        reason
    })
}

/// Removes the sounds of `plays` from the mixers of `targets` that hold them
fn stopped(targets: &mut [Target<'_>], plays: Vec<Play>) {
    plays
        .into_iter()
        .filter_map(|play| play.place)
        .for_each(|place| {
            targets
                .iter_mut()
                .filter(|target| target.key == place.key)
                .for_each(|target| target.mixer.remove(place.id));
        });
}

/// Opens `file` on `target` as far into `clip` as `played` seconds
pub(super) fn opened(
    target: &mut Target<'_>,
    file: &Path,
    active: &Active,
    clip: Clip,
    played: f64,
) -> Result<Play, String> {
    let data = StreamingSoundData::from_file(file).map_err(crate::back::audio::reason)?;
    let data = clipped(data, clip, played, active.level);
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

/// Returns whether a sound of `clip` loops once `played` seconds of the clip
/// have played, which a clip does until its last pass through its loop
/// begins, once a devamp cue has let it go on
pub(super) fn circles(clip: Clip, played: f64) -> bool {
    clip.looping.is_some() && clip.last_pass().is_none_or(|last| played < last)
}

/// Returns `data` set to play `clip` from where it is once `played` seconds
/// of it have played, and at `level`.  A clip let go on from its loop that
/// has begun its last pass gets no loop, so that its sound plays on from the
/// end of its loop.
pub(in crate::back::audio) fn clipped(
    data: StreamingSoundData<FromFileError>,
    clip: Clip,
    played: f64,
    level: f32,
) -> StreamingSoundData<FromFileError> {
    // Cut from the start of the file, so that times in the cut are times in
    // the file, which kira seeks to directly.  A cut is left out where it
    // would end past the last frame of the file.
    let cut = (clip.end < data.duration().as_secs_f64()).then(|| Region::from(0.0..clip.end));
    let looped = clip
        .looping
        .filter(|_| circles(clip, played))
        .map(|section| Region {
            start: section.start.into(),
            end: if section.end < clip.end {
                EndPosition::Custom(section.end.into())
            } else {
                EndPosition::EndOfAudio
            },
        });
    data.slice(cut)
        .loop_region(looped)
        .start_position(clip.position(played))
        .volume(level)
}
