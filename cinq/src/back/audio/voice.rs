// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sound of one run of an audio or video cue
//!
//! The run plays once on each device that its outputs are on, each sound in
//! a kira manager of its own, whose renderer the mixer of the device
//! drives, as [`super::feed`] tells.  The clock of the run follows the
//! sound on the device of the first output that the cue sends to.  Sounds
//! on two devices keep the clocks of their devices, which drift apart
//! unless the devices share a clock.

use std::{collections::HashMap, path::Path, time::Duration};

use kira::{
    AudioManager, AudioManagerSettings, Capacities, Tween,
    sound::{
        FromFileError, PlaybackState, Region, SoundData,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
    track::MainTrackBuilder,
};

use super::{
    feed::Feed,
    mixer::{Gains, Mixer},
    patch::Key,
    preload::Preloaded,
};
use crate::back::{Active, Clip, Command, TICK};

/// Time a sound takes to reach a level the workspace asks for, which
/// outlasts a tick so that fades move smoothly from one tick to the next
const GLIDE: Duration = TICK.saturating_mul(2);

/// Mixers of the devices, by their keys, or why each could not open
pub(super) type Mixers = HashMap<Key, Result<Mixer, String>>;

pub(super) struct Voice {
    pub(super) run: u32,
    /// Sound of the run on each device that it plays on
    plays: Vec<Play>,
    /// Level last sent to the sounds
    level: f32,
    /// Pause state last sent to the sounds
    paused: bool,
}

struct Play {
    sound: StreamingSoundHandle<FromFileError>,
    /// Place of the sound in the mixer of its device, which a test that
    /// plays the sound in a manager of its own has none of
    place: Option<Place>,
}

struct Place {
    key: Key,
    /// Identity of the sound in the mixer
    id: u64,
    gains: Gains,
    /// Manager of the sound alone, which hands the sound its commands
    _manager: AudioManager<Feed>,
}

/// Mixer that a sound of a run starts in, with the gains it plays at
pub(super) struct Target<'a> {
    pub(super) key: Key,
    pub(super) mixer: &'a mut Mixer,
    pub(super) gains: Gains,
}

impl Voice {
    /// Opens the file of a run on each target and starts its sound at the
    /// level of the run, as far into its clip as the run has played, since
    /// the clock may have passed the start by part of a tick.  The sound
    /// streams from the file, so long files start as quickly as short ones,
    /// but opening the file can wait on the disk, which [`Voice::start`]
    /// avoids.  Returns why the sound cannot start, as when its file has
    /// gone since it loaded or a mixer has more to do than it can take.
    pub(super) fn open(
        targets: Vec<Target<'_>>,
        file: &Path,
        active: &Active,
        clip: Clip,
    ) -> Result<Self, String> {
        let plays = targets
            .into_iter()
            .map(|target| opened(target, file, active, clip))
            .collect::<Result<_, _>>()?;
        Ok(Self::with(plays, active))
    }

    /// Starts a sound made ready ahead on the first target, which waits at
    /// the start of its clip, at the level that the run has now, and opens
    /// the file on the others.  A run that the clock started between two
    /// ticks, as a pre-wait or a chain does, has played part of a tick
    /// already, and its sound starts late by that part: kira keeps what it
    /// has decoded ahead when a playing sound seeks, so seeking would play
    /// the start twice.  Returns why the sound cannot start, as when a
    /// mixer has more to do than it can take.
    pub(super) fn start(
        targets: Vec<Target<'_>>,
        preloaded: Preloaded,
        file: &Path,
        active: &Active,
        clip: Clip,
    ) -> Result<Self, String> {
        let mut targets = targets.into_iter();
        let first = targets
            .next()
            .map(|target| {
                let mut play = placed(target, preloaded)?;
                let now = Tween {
                    duration: Duration::ZERO,
                    ..Tween::default()
                };
                play.sound.set_volume(active.level, now);
                Ok::<_, String>(play)
            })
            .transpose()?;
        let others = targets
            .map(|target| opened(target, file, active, clip))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::with(
            first.into_iter().chain(others).collect(),
            active,
        ))
    }

    #[cfg(test)]
    pub(super) fn of(sound: StreamingSoundHandle<FromFileError>, active: &Active) -> Self {
        Self::with(vec![Play { sound, place: None }], active)
    }

    fn with(plays: Vec<Play>, active: &Active) -> Self {
        Self {
            run: active.run,
            plays,
            level: active.level,
            paused: false,
        }
    }

    /// Returns the devices that the run plays on, in order
    pub(super) fn keys(&self) -> Vec<&Key> {
        self.plays
            .iter()
            .filter_map(|play| play.place.as_ref().map(|place| &place.key))
            .collect()
    }

    /// Sends the sounds any change in the level or pause state of their
    /// run
    pub(super) fn followed(self, active: &Active) -> Self {
        let plays = self
            .plays
            .into_iter()
            .map(|mut play| {
                match (self.paused, active.paused) {
                    (false, true) => play.sound.pause(Tween::default()),
                    (true, false) => play.sound.resume(Tween::default()),
                    _ => {}
                }
                if self.level != active.level {
                    let glide = Tween {
                        duration: GLIDE,
                        ..Tween::default()
                    };
                    play.sound.set_volume(active.level, glide);
                }
                play
            })
            .collect();
        Self {
            plays,
            level: active.level,
            paused: active.paused,
            ..self
        }
    }

    /// Ramps each sound to the gains that `gains` give its device, where
    /// they have changed, as when a send or an output changes while the
    /// run plays
    pub(super) fn routed(self, gains: &[(Key, Gains)], mixers: &mut Mixers) -> Self {
        let plays = self
            .plays
            .into_iter()
            .map(|play| {
                let Some(place) = play.place else {
                    return play;
                };
                let wanted = gains.iter().find(|(key, _)| *key == place.key);
                let place = match (wanted, mixers.get_mut(&place.key)) {
                    (Some((_, gains)), Some(Ok(mixer))) if *gains != place.gains => {
                        mixer.route(place.id, gains.clone());
                        Place {
                            gains: gains.clone(),
                            ..place
                        }
                    }
                    _ => place,
                };
                Play {
                    place: Some(place),
                    ..play
                }
            })
            .collect();
        Self { plays, ..self }
    }

    /// Returns the command that tells the workspace how far the sound of the
    /// run has played on the first device that `plays` says still plays, as
    /// long as the sound is still playing.  The position of the sound is
    /// where the audio device took it at the start of its last buffer,
    /// which kira shares without waiting on the device.  A run whose every
    /// device has stopped goes on by the clock of the show, since its sound
    /// would hold it still.
    pub(super) fn heard(&self, active: &Active, plays: impl Fn(&Key) -> bool) -> Option<Command> {
        let clip = active.sound()?;
        let sound = &self
            .plays
            .iter()
            .find(|play| play.place.as_ref().is_none_or(|place| plays(&place.key)))?
            .sound;
        (sound.state() != PlaybackState::Stopped).then(|| Command::Heard {
            cue: active.id,
            run: active.run,
            played: clip.played_at(sound.position(), active.played()),
        })
    }

    /// Stops the sounds, which their mixers fade out as quickly as kira's
    /// default fade, short enough to sound immediate and long enough to
    /// avoid a click
    pub(super) fn stop(self, mixers: &mut Mixers) {
        self.plays
            .into_iter()
            .filter_map(|play| play.place)
            .for_each(|place| {
                if let Some(Ok(mixer)) = mixers.get_mut(&place.key) {
                    mixer.remove(place.id);
                }
            });
    }
}

fn opened(target: Target<'_>, file: &Path, active: &Active, clip: Clip) -> Result<Play, String> {
    let data = StreamingSoundData::from_file(file).map_err(super::reason)?;
    let data = clipped(data, clip, clip.position(active.played()), active.level);
    placed(target, data)
}

/// Plays `data` in a manager of its own, whose renderer the mixer of
/// `target` takes and mixes at the gains of `target`
fn placed<D>(target: Target<'_>, data: D) -> Result<Play, String>
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
            key: target.key,
            id,
            gains: target.gains,
            _manager: manager,
        }),
    })
}

/// Returns `data` set to play `clip`, from `position` in the file and at
/// `level`.
pub(super) fn clipped(
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
