// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! How edits and fades move the level and the clip of a fired cue

use super::{Active, Effect, Fade};
use crate::back::{CueId, LOUDEST};

impl Active {
    pub(in crate::back::workspace::running) fn faded_out(&self) -> bool {
        self.fade
            .is_some_and(|fade| fade.then_stop && fade.remaining <= 0.0)
    }

    /// Returns the run with its level moved by `by` dB, as far up as
    /// [`LOUDEST`] and without a limit down, and with any fade left to go on
    /// from there
    pub(in crate::back::workspace) fn moved(self, by: f32) -> Self {
        Self {
            level: (self.level + by).min(LOUDEST),
            ..self
        }
    }

    /// Returns the run with the fades that the fade cue `by` is making on
    /// it, and the fade that it has yet to make if it is a run of that cue,
    /// ending at `level`.  A fade under way keeps its remaining time, so
    /// that the level turns from where it is toward the new one.
    pub(in crate::back::workspace) fn retargeted(self, by: CueId, level: f32) -> Self {
        let to = |fade: Fade| Fade { to: level, ..fade };
        Self {
            fade: self
                .fade
                .map(|fade| if fade.by == Some(by) { to(fade) } else { fade }),
            effect: match self.effect {
                Effect::Fade { target, fade } if self.id == by && !self.started => Effect::Fade {
                    target,
                    fade: to(fade),
                },
                effect => effect,
            },
            ..self
        }
    }

    /// Returns the run with its clip let go on from its loop, as
    /// [`Clip::released`](crate::back::Clip::released) tells, `notice`
    /// seconds ahead at the least, and with its action lasting until the
    /// clip ends.  A run whose clip stays as it was keeps its length.
    pub(in crate::back::workspace::running) fn released(self, notice: f64) -> Self {
        let effect = match self.effect {
            Effect::Sound(clip) => Effect::Sound(clip.released(self.played(), notice)),
            Effect::Video { clip, sound } => Effect::Video {
                clip: clip.released(self.played(), notice),
                sound,
            },
            effect => effect,
        };
        match effect.clip().and_then(|clip| clip.lasts()) {
            Some(length) if effect != self.effect => Self {
                effect,
                length,
                ..self
            },
            _ => self,
        }
    }

    pub(in crate::back::workspace::running) fn under(self, fade: Fade) -> Self {
        Self {
            fade: Some(Fade {
                opacity: self.opacity(),
                ..fade
            }),
            ..self
        }
    }

    /// Moves the level `seconds` along its fade.  Each step covers the share
    /// of the remaining change that the step is of the remaining time, so
    /// the level moves in a straight line in dB.  A fade that leaves the cue
    /// running is dropped once it ends.
    pub(in crate::back::workspace::running) fn faded(self, seconds: f64) -> Self {
        match self.fade {
            Some(fade) if fade.remaining > seconds => Self {
                level: self.level + (fade.to - self.level) * (seconds / fade.remaining) as f32,
                fade: Some(Fade {
                    remaining: fade.remaining - seconds,
                    ..fade
                }),
                ..self
            },
            Some(fade) => Self {
                level: fade.to,
                fade: fade.then_stop.then_some(Fade {
                    remaining: 0.0,
                    ..fade
                }),
                ..self
            },
            None => self,
        }
    }
}
