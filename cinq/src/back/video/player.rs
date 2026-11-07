// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures of the clip of a video cue, which follow the clock of its run,
//! or the picture of a picture cue, which shows as soon as its run does

use std::{
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{Receiver, TryRecvError},
    },
};

use super::{
    decoder::{self, Frame, Picture, Sent, UNDECODABLE},
    still,
};
use crate::back::Shown;

/// Player moved on to the clock of its run, along with the newest picture
/// that has come due, if one has, or why the picture cannot play
pub(super) type Advanced = Result<(Player, Option<Picture>), String>;

/// Pictures of the clip of a video cue, which a thread of its own decodes
/// for each pass through the clip, a few pictures ahead of the clock of the
/// run that shows them, so that the first ones wait ready until the run
/// starts its clock.  The pass after the one under way of a clip that loops
/// decodes ahead as well, so that it has decoded from the sync sample
/// before the start of the clip by the time the pass under way ends.  The
/// threads stop decoding once the player goes.  The picture of a picture
/// cue makes a single pass of one picture, which shows until the run ends.
pub(super) struct Player {
    path: PathBuf,
    shown: Shown,
    pass: u32,
    /// Pictures of the pass under way, each with the time of the run at
    /// which it is due, until the clip ends, as one that loops never does
    pictures: Option<Receiver<Sent>>,
    /// Pictures of the pass after it, for a clip that loops
    following: Option<Receiver<Sent>>,
    /// Picture that arrived ahead of its time, which waits for the clock
    /// of the run to reach it
    next: Option<Frame>,
    /// Life of the player, which the threads of its passes watch, so that
    /// they stop once it has gone
    life: Arc<()>,
}

impl Player {
    /// Starts decoding what the stage window shows of the file at `path`:
    /// a clip of a video file, or a picture file
    pub(super) fn start(path: PathBuf, shown: Shown) -> Self {
        let life = Arc::new(());
        let (pictures, following) = match shown {
            Shown::Clip(clip) => {
                let decoding =
                    |pass| decoder::start(path.clone(), clip, pass, Arc::downgrade(&life));
                (decoding(0), clip.looping.then(|| decoding(1)))
            }
            Shown::Still => (still::start(path.clone()), None),
        };
        Self {
            path,
            shown,
            pass: 0,
            pictures: Some(pictures),
            following,
            next: None,
            life,
        }
    }

    /// Returns the player once the clock of its run has reached `played`
    /// seconds of its clip, along with the newest picture that has come due
    /// since, if one has, or why the picture cannot play.  Pictures that
    /// came due along with a newer one are let go unseen, so that a picture
    /// that arrives late never holds back those after it.
    pub(super) fn advanced(self, played: f64) -> Advanced {
        self.advanced_from(played, None)
    }

    /// Returns the seconds of its clip at which the next picture comes due,
    /// when it has arrived ahead of its time
    pub(super) fn next_due(&self) -> Option<f64> {
        self.next.as_ref().map(|frame| frame.at)
    }

    /// Goes on with [`Player::advanced`], which has found `due` due so far
    fn advanced_from(self, played: f64, due: Option<Frame>) -> Advanced {
        let arrived = match (self.next, &self.pictures) {
            (Some(frame), _) => Some(Sent::Picture(frame)),
            (None, Some(pictures)) => match pictures.try_recv() {
                Ok(sent) => Some(sent),
                Err(TryRecvError::Empty) => None,
                // Thread gone without ending its pass, as one that panics
                Err(TryRecvError::Disconnected) => Some(Sent::Failed(UNDECODABLE.to_owned())),
            },
            (None, None) => None,
        };
        let player = Self { next: None, ..self };
        match arrived {
            Some(Sent::Picture(frame)) if frame.at <= played => {
                player.advanced_from(played, Some(frame))
            }
            Some(Sent::Picture(frame)) => Ok((
                Self {
                    next: Some(frame),
                    ..player
                },
                due.map(|frame| frame.picture),
            )),
            Some(Sent::Ended) => player.passed().advanced_from(played, due),
            Some(Sent::Failed(reason)) => Err(reason),
            None => Ok((player, due.map(|frame| frame.picture))),
        }
    }

    /// Returns the player once the pass under way has ended: for a clip
    /// that loops, playing the pass after it while the one after that
    /// decodes ahead, and for any other, or a picture, with no pass left to
    /// play
    fn passed(self) -> Self {
        match (self.following, self.shown) {
            (Some(following), Shown::Clip(clip)) => {
                let pass = self.pass + 1;
                Self {
                    pictures: Some(following),
                    following: Some(decoder::start(
                        self.path.clone(),
                        clip,
                        pass + 1,
                        Arc::downgrade(&self.life),
                    )),
                    pass,
                    ..self
                }
            }
            _ => Self {
                pictures: None,
                following: None,
                ..self
            },
        }
    }
}
