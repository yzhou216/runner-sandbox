// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Video output, where the back end has side effects
//!
//! [`Video::sync`] runs after every change to the workspace, as
//! [`super::Audio::sync`] does, and makes the pictures match it.  Each run
//! of a video cue gets a thread of its own from the moment GO fires it,
//! which decodes the clip of the run a few pictures ahead of the clock of
//! the run, and a run whose clip loops gets a second one, which decodes the
//! next pass through the clip before the pass under way ends.  Each sync
//! hands on the newest picture that the clock of each run has reached, so
//! that the picture follows the clock of the show and holds still while
//! the run is paused.  The sound follows the clock of the audio device
//! instead, from which the clock of the show drifts over a long video, as
//! cinq-show(5) tells.  The threads of a run that ends go with it.  Ahead
//! of that, the pictures that the next GO of each list may start are made
//! ready, as their sounds are, since decoding has to start at the sync
//! sample before the start of a clip, a few seconds before it in many
//! files.  The workspace stays the only record of what should be seen, so
//! a picture that cannot play goes back to it as a file that cannot be
//! played.  Video files hold H.264 in MP4, whose tracks [`track`] reads,
//! and which [`decoder`] decodes with Cisco's openh264.

mod decoder;
mod player;
#[cfg(test)]
pub(super) mod tests;
mod track;

use std::collections::HashMap;

use itertools::{Either, Itertools};

use super::{Active, Command, Cue, CueId, Kind, Media, Problem, Wanted, Workspace};
pub use decoder::Picture;
use player::{Advanced, Player};
pub(super) use track::probe;

/// Endings of the names of the video files that the video output plays,
/// as MP4, QuickTime, and iTunes name them
pub const VIDEO_FILES: [&str; 3] = ["m4v", "mov", "mp4"];

/// Video output for a workspace
#[derive(Default)]
pub struct Video {
    /// Pictures of the runs of video cues that GO has fired, by cue, each
    /// with the serial number of its run
    runs: HashMap<CueId, (u32, Player)>,
    /// Pictures that the next GO may start, made ready ahead, by cue, each
    /// with the part of a file that it plays
    ready: HashMap<CueId, (Wanted, Player)>,
}

impl Video {
    /// Makes the pictures match the workspace.  Returns the workspace with
    /// the files whose picture cannot play, along with the pictures that
    /// have come due since the last sync, each with its cue.
    pub fn sync(&mut self, workspace: Workspace) -> (Workspace, Vec<(CueId, Picture)>) {
        // Players whose run has ended, and those made ready that no run
        // takes, stay behind and go at the end
        let mut ended = std::mem::take(&mut self.runs);
        let mut ready = std::mem::take(&mut self.ready);
        let advanced: Vec<(&Cue, u32, Advanced)> = workspace
            .active()
            .iter()
            .filter_map(|active| {
                let cue = workspace.cue(active.id)?;
                let clip = active.video()?;
                let wanted = workspace.path(cue).map(|path| Wanted { path, clip });
                // Player of the run, or else the one made ready for it
                let player = ended
                    .remove(&cue.id)
                    .filter(|(run, _)| *run == active.run)
                    .map(|(_, player)| player)
                    .or_else(|| {
                        ready
                            .remove(&cue.id)
                            .filter(|(made, _)| Some(made) == wanted.as_ref())
                            .map(|(_, player)| player)
                    });
                Some((cue, active.run, moved_on(wanted, active, player)))
            })
            .collect();
        let (playing, failures): (Vec<_>, Vec<_>) =
            advanced
                .into_iter()
                .partition_map(|(cue, run, advanced)| match advanced {
                    Ok((player, picture)) => {
                        Either::Left(((cue.id, (run, player)), (cue.id, picture)))
                    }
                    Err(reason) => Either::Right(Command::Loaded {
                        cue: cue.id,
                        file: cue.file.clone(),
                        media: Media::Failed(reason),
                    }),
                });
        let (runs, pictures): (HashMap<_, _>, Vec<_>) = playing.into_iter().unzip();
        self.runs = runs;
        self.ready = workspace
            .upcoming()
            .filter(|(cue, _)| cue.kind == Kind::Video)
            .map(|(cue, wanted)| {
                let player = ready
                    .remove(&cue.id)
                    .filter(|(made, _)| *made == wanted)
                    .map_or_else(
                        || Player::start(wanted.path.clone(), wanted.clip),
                        |(_, player)| player,
                    );
                (cue.id, (wanted, player))
            })
            .collect();
        let pictures = pictures
            .into_iter()
            .filter_map(|(cue, picture)| Some((cue, picture?)))
            .collect();
        (
            failures.into_iter().fold(workspace, Workspace::apply),
            pictures,
        )
    }

    /// Returns how many seconds of the show clock pass before the next
    /// picture of a video cue that plays and is not paused comes due, of
    /// the pictures that have arrived ahead of their time, so that the
    /// interface can wake to show it then
    pub fn next_due(&self, workspace: &Workspace) -> Option<f64> {
        self.runs
            .iter()
            .filter_map(|(&cue, (run, player))| {
                let active = workspace
                    .active_of(cue)
                    .filter(|active| active.run == *run && !active.paused)
                    .filter(|active| active.shows_picture())?;
                Some(player.next_due()? - active.played())
            })
            .min_by(f64::total_cmp)
    }
}

/// Returns the player of the run `active` of a video cue that plays
/// `wanted`, moved on to the clock of the run once the pre-wait is over:
/// `player`, the player of the run or the one made ready for it, or else
/// one that starts now
fn moved_on(wanted: Option<Wanted>, active: &Active, player: Option<Player>) -> Advanced {
    let player = match player {
        Some(player) => player,
        None => {
            let wanted = wanted.ok_or_else(|| Problem::NoFile.to_string())?;
            Player::start(wanted.path, wanted.clip)
        }
    };
    if active.shows_picture() {
        player.advanced(active.played())
    } else {
        Ok((player, None))
    }
}
