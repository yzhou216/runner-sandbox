// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures of the clip of a video cue, which follow the clock of its run,
//! or the picture of a picture cue, which shows as soon as its run does

use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        mpsc::{Receiver, TryRecvError},
    },
};

use super::{
    decoder::{self, Frame, Picture, Sent, UNDECODABLE},
    still,
};
use crate::back::{Clip, Pass, Shown};

/// Player moved on to the clock of its run, with the newest picture that has
/// come due, if any, or why the picture cannot play
pub(super) type Advanced = Result<(Player, Option<Picture>), String>;

/// Pictures of the clip of a video cue, which a thread of its own decodes
/// for each pass through the clip, a few pictures ahead of the clock of the
/// run, so that the first ones wait ready until the run starts its clock.
/// The pass after the one under way decodes ahead as well, so that it has
/// decoded from the sync sample before the start of its stretch by the time
/// the pass under way ends.  A clip that loops has one until a devamp cue
/// lets it go on, and the player ends with the last pass.  The picture of a
/// picture cue makes a single pass of one picture, which shows until the run
/// ends.
pub(super) struct Player {
    path: PathBuf,
    shown: Shown,
    pass: u32,
    /// Pictures of the pass under way, each with the time of the run at
    /// which it is due, until the last pass has ended
    pictures: Option<Receiver<Sent>>,
    /// Pictures of the pass after it, along with the pass they decode, when
    /// the clip has one
    following: Option<(Pass, Receiver<Sent>)>,
    /// Picture that arrived ahead of its time, which waits for the clock
    /// of the run to reach it
    next: Option<Frame>,
    /// Life of the player, which the threads of its passes watch, so that
    /// they stop once it has gone
    life: Arc<()>,
}

impl Player {
    pub(super) fn start(path: PathBuf, shown: Shown) -> Self {
        let life = Arc::new(());
        let (pictures, following) = match shown {
            Shown::Clip(clip) => (
                decoding(&path, &clip, 0, &life).map(|(_, pictures)| pictures),
                decoding(&path, &clip, 1, &life),
            ),
            Shown::Still => (Some(still::start(path.clone())), None),
        };
        Self {
            path,
            shown,
            pass: 0,
            pictures,
            following,
            next: None,
            life,
        }
    }

    /// Returns the player once the clock of its run has reached `played`
    /// seconds of its clip, as [`Advanced`] tells.  Pictures that came due
    /// along with a newer one are let go unseen, so that a picture that
    /// arrives late never holds back those after it.
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

    /// Returns the player once the pass under way has ended, on the pass
    /// after it, which is the last one when the clip has no more
    fn passed(self) -> Self {
        let following = match self.shown {
            Shown::Clip(clip) => decoding(&self.path, &clip, self.pass + 2, &self.life),
            Shown::Still => None,
        };
        Self {
            pictures: self.following.map(|(_, pictures)| pictures),
            following,
            pass: self.pass + 1,
            ..self
        }
    }

    /// Returns the player once the clip of its run has changed to `shown`,
    /// as a devamp cue changes it.  The pass under way goes on, and so does
    /// the pass after it where the new clip gives the same one, while another
    /// replaces it, with none left where the new clip has no more.  A picture
    /// of the pass that goes has not reached the player yet, since the player
    /// takes pictures from the pass under way alone, so none of it shows.
    pub(super) fn released(self, shown: Shown) -> Self {
        let following = match shown {
            Shown::Clip(clip) if shown != self.shown => {
                let pass = clip.pass(self.pass + 1);
                match self.following {
                    kept @ Some((decoded, _)) if Some(decoded) == pass => kept,
                    _ => decoding(&self.path, &clip, self.pass + 1, &self.life),
                }
            }
            _ => self.following,
        };
        Self {
            shown,
            following,
            ..self
        }
    }
}

/// Starts decoding pass `index` through `clip` for the player whose life is
/// `life`, if the clip has one, and returns the way to its pictures along
/// with the pass
fn decoding(
    path: &Path,
    clip: &Clip,
    index: u32,
    life: &Arc<()>,
) -> Option<(Pass, Receiver<Sent>)> {
    clip.pass(index).map(|pass| {
        (
            pass,
            decoder::start(path.to_owned(), pass, Arc::downgrade(life)),
        )
    })
}
