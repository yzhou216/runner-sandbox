// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! How the run of a voice learns from its sound: how far the sound has
//! played, and whether it has stopped early

use kira::sound::{FromFileError, PlaybackState, streaming::StreamingSoundHandle};

use super::Voice;
use crate::back::{Active, Clip, Command, audio::patch::Key};

/// Seconds of its file that a sound may leave unplayed and still have ended
/// as it should, since the position of a sound is where its device took it
/// as its last buffer began, which lies a buffer short of its end
const ENDED: f64 = 1.0;

impl Voice {
    /// Returns the command that tells the workspace how far the sound of the
    /// run has played on the first device that `plays` says still plays, as
    /// long as the sound is still playing.  The position of the sound is
    /// where the audio device took it at the start of its last buffer,
    /// which kira shares without waiting on the device.  A run whose every
    /// device has stopped goes on by the clock of the show, since its sound
    /// would hold it still.
    pub(in crate::back::audio) fn heard(
        &self,
        active: &Active,
        plays: impl Fn(&Key) -> bool,
    ) -> Option<Command> {
        let clip = active.sound()?;
        let sound = self.leading(plays)?;
        (sound.state() != PlaybackState::Stopped).then(|| Command::Heard {
            cue: active.id,
            run: active.run,
            played: clip.played_at(sound.position(), active.played()),
        })
    }

    /// Returns whether the sound of the run has stopped before its end,
    /// which it does when its file ends sooner than the header said, or
    /// cannot be read on, while its device plays on.  A sound whose clip
    /// loops has no end to reach, so that any stop is early, and others stop
    /// early when more than [`ENDED`] seconds of the file lie ahead of them,
    /// `ends` being where in the file the sound should end.  A device that
    /// stalls or fails keeps its sounds as they were, and a sound that the
    /// operator, a panic, or a fade ends leaves its voice before this looks,
    /// so that none of them count.
    pub(in crate::back::audio) fn cut_short(
        &self,
        clip: Clip,
        ends: f64,
        plays: impl Fn(&Key) -> bool,
    ) -> bool {
        self.leading(plays).is_some_and(|sound| {
            sound.state() == PlaybackState::Stopped
                && (clip.lasts().is_none() || ends - sound.position() > ENDED)
        })
    }

    /// Returns the first sound of the run whose device `plays` says still
    /// plays, which the clock of the run follows
    fn leading(
        &self,
        plays: impl Fn(&Key) -> bool,
    ) -> Option<&StreamingSoundHandle<FromFileError>> {
        self.plays
            .iter()
            .find(|play| play.place.as_ref().is_none_or(|place| plays(&place.key)))
            .map(|play| &play.sound)
    }
}
