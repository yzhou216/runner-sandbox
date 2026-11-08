// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Video output, where the back end has side effects
//!
//! [`Video::sync`] runs after every change to the workspace, as
//! [`super::Audio::sync`] does, and makes the pictures match it.  Each run
//! of a video or picture cue gets a [`Player`] from the moment GO fires it,
//! and each sync hands on the newest picture that the clock of each run has
//! reached, so that the picture follows the clock of the show and holds
//! still while the run is paused.  The sound follows the clock of the audio
//! device instead, from which the clock of the show drifts over a long
//! video, as cinq-show(5) tells.  The pictures that the next GO of each
//! list may start are made ready ahead, as their sounds are, since decoding
//! has to start at the sync sample before the start of a clip, a few
//! seconds before it in many files, and a large picture file takes a moment
//! to decode.  The workspace stays the only record of what should be seen,
//! so a picture that cannot play goes back to it as a file that cannot be
//! played.  Video files hold H.264 in MP4, whose tracks [`track`] reads,
//! and which [`decoder`] decodes with rust_h264, while picture files hold
//! PNG or JPEG, which [`still`] decodes.

mod decoder;
mod player;
mod still;
#[cfg(test)]
pub(super) mod tests;
mod track;

use std::{collections::HashMap, io, path::PathBuf};

use itertools::{Either, Itertools};

use super::{Active, Command, CueId, Media, Problem, Shown, Workspace};
pub use decoder::Picture;
use player::{Advanced, Player};
pub(super) use still::probe as probe_still;
pub(super) use track::probe;

/// Endings of the names of the video files that the video output plays,
/// as MP4, QuickTime, and iTunes name them
pub const VIDEO_FILES: [&str; 3] = ["m4v", "mov", "mp4"];

/// Endings of the names of the picture files that the video output shows,
/// as PNG and JPEG name them
pub const PICTURE_FILES: [&str; 3] = ["jpeg", "jpg", "png"];

/// What a player decodes, which one made ready ahead has to match for a
/// run to take it
#[derive(Debug, Clone, PartialEq)]
struct Showing {
    path: PathBuf,
    shown: Shown,
}

#[derive(Default)]
pub struct Video {
    /// Players of the runs, each with the serial number of its run
    runs: HashMap<CueId, (u32, Player)>,
    /// Players made ready ahead for the next GO
    ready: HashMap<CueId, (Showing, Player)>,
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
        let ((runs, pictures), failures): ((HashMap<_, _>, Vec<_>), Vec<_>) = workspace
            .active()
            .iter()
            .filter_map(|active| {
                let cue = workspace.cue(active.id)?;
                let shown = active.shown()?;
                let showing = workspace.path(cue).map(|path| Showing { path, shown });
                // Player of the run, or else the one made ready for it
                let player = ended
                    .remove(&cue.id)
                    .filter(|(run, _)| *run == active.run)
                    .map(|(_, player)| player)
                    .or_else(|| {
                        ready
                            .remove(&cue.id)
                            .filter(|(made, _)| Some(made) == showing.as_ref())
                            .map(|(_, player)| player)
                    });
                Some((cue, active.run, moved_on(showing, active, player)))
            })
            .partition_map(|(cue, run, advanced)| match advanced {
                Ok((player, picture)) => Either::Left((
                    (cue.id, (run, player)),
                    picture.map(|picture| (cue.id, picture)),
                )),
                Err(reason) => Either::Right(Command::Loaded {
                    cue: cue.id,
                    file: cue.file.clone(),
                    media: Media::Failed(reason),
                }),
            });
        self.runs = runs;
        self.ready = workspace
            .upcoming()
            .filter_map(|(cue, path)| {
                let showing = Showing {
                    path,
                    shown: cue.shown()?,
                };
                let player = ready
                    .remove(&cue.id)
                    .filter(|(made, _)| *made == showing)
                    .map_or_else(
                        || Player::start(showing.path.clone(), showing.shown),
                        |(_, player)| player,
                    );
                Some((cue.id, (showing, player)))
            })
            .collect();
        (
            failures.into_iter().fold(workspace, Workspace::apply),
            pictures.into_iter().flatten().collect(),
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

/// Returns the player of the run `active` of a video or picture cue that
/// shows `showing`, moved on to the clock of the run once the pre-wait is
/// over: `player`, the player of the run or the one made ready for it, or
/// else one that starts now
fn moved_on(showing: Option<Showing>, active: &Active, player: Option<Player>) -> Advanced {
    let player = player
        .or_else(|| showing.map(|showing| Player::start(showing.path, showing.shown)))
        .ok_or_else(|| Problem::NoFile.to_string())?;
    if active.shows_picture() {
        player.advanced(active.played())
    } else {
        Ok((player, None))
    }
}

/// Tells why a video or picture file cannot be read, in the words that the
/// sound output uses for a sound file
fn unreadable(error: io::Error) -> String {
    match error.kind() {
        io::ErrorKind::NotFound => "file not found".to_owned(),
        _ => error.to_string(),
    }
}
