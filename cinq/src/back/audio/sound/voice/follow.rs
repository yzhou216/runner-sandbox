// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! How the sounds of a run follow it: its level, its pause state, its
//! loop, and its routing

use std::time::Duration;

use kira::Tween;

use super::{
    Mixers, Voice,
    place::{Place, Play},
};
use crate::back::{
    Active, TICK,
    audio::patch::{Gains, Key},
};

/// Time a sound takes to reach a level the workspace asks for, which
/// outlasts a tick so that fades move smoothly from one tick to the next
const GLIDE: Duration = TICK.saturating_mul(2);

impl Voice {
    /// Sends the sounds any change in the level or pause state of their
    /// run, and takes away their loop once their run, which a devamp cue
    /// has let go on, begins its last pass through the loop.  Kira decodes a
    /// sound about 16384 frames ahead of where it plays and applies a new
    /// loop as it decodes, so the sound leaves the loop at the end of the
    /// pass that the decoder is in.  Before the last pass begins, the
    /// decoder may still be in the pass before it, and the sound would leave
    /// the loop a pass too early.  After the last pass has begun, the
    /// decoder is in it, unless the pass is shorter than the sound decodes
    /// ahead, and the sound leaves the loop at its end, as planned.  The
    /// clock of the run lies a buffer of the device ahead of its sound at
    /// most, far less than the sound decodes ahead, so the loop goes the
    /// moment that the clock reaches the start of the last pass.
    pub(in crate::back::audio) fn followed(self, active: &Active) -> Self {
        let leaving = self.looped
            && active
                .sound()
                .is_some_and(|clip| clip.last_pass().is_some_and(|last| active.played() >= last));
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
                if leaving {
                    play.sound.set_loop_region(None);
                }
                play
            })
            .collect();
        Self {
            plays,
            level: active.level,
            paused: active.paused,
            looped: self.looped && !leaving,
            ..self
        }
    }

    /// Ramps each sound to the gains that `gains` give its device, where
    /// they have changed, as when a send or an output changes while the
    /// run plays
    pub(in crate::back::audio::sound) fn routed(
        self,
        gains: &[(Key, Gains)],
        mixers: &mut Mixers,
    ) -> Self {
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
}
