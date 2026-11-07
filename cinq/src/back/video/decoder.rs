// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Threads that decode the clip of one run of a video cue, a pass through
//! the clip each, a few pictures ahead of the clock of the run
//!
//! A pass starts a decoder at the last sync sample before the start of the
//! clip that shows no later than the first picture of the clip, and decodes
//! from there to the last sample that shows before its end.  Each sample
//! holds a picture, which the decoder finishes as soon as it has read the
//! sample rather than once the next picture starts, since a sample that
//! fails to decode would otherwise take the picture before it along.  The
//! pictures go out in the order they show, which [`order`] keeps, each at
//! the time of its own sample, and those before the start of the clip are
//! let go without turning them into RGBA, which [`color`] turns the rest
//! into.  A pass ends by telling its player that it has sent every
//! picture, or why its picture cannot play, as a pass that decodes no
//! picture cannot, and it stops decoding as soon as its player has gone.
//! Each pass opens its file and reads the index of its track anew, which
//! takes a long file a moment, hidden for a clip that loops by the pass
//! that decodes ahead.

mod color;
mod order;

use std::{
    path::{Path, PathBuf},
    sync::{
        Weak,
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
};

use rust_h264::{
    decoder::{ChromaFormat, Decoder, Frame as Decoded},
    nal::NalUnit,
};

use super::track::Track;
use crate::back::Clip;
pub use color::Picture;
use color::rgba;
use order::Order;

/// Pictures that the thread of a pass decodes ahead of the clock of its
/// run, besides the one it holds as it waits to send it: enough to ride out
/// the pictures that take longest to decode, such as those that start a
/// group of pictures, and few enough to keep the memory of a pass small,
/// as each picture of 1080p takes 8 MB
const AHEAD: usize = 2;

/// Why the picture of a file that holds H.264 cannot play, when the decoder
/// decodes none of it
pub(super) const UNDECODABLE: &str = "its picture cannot be decoded";

/// Picture along with the time of its run at which it is due: the seconds
/// of the clip played, counting every pass of a clip that loops
pub(super) struct Frame {
    pub(super) at: f64,
    pub(super) picture: Picture,
}

/// What the thread of a pass sends its player
pub(super) enum Sent {
    Picture(Frame),
    /// Pass is over, and every picture of it has been sent
    Ended,
    Failed(String),
}

/// Why decoding stops before the end of the pass
enum Stop {
    /// Player of the run has gone, along with the way to it
    Gone,
    /// File cannot be read, for the given reason
    Failed(String),
}

/// Starts decoding pass `pass` through `clip` of the video file at `path`
/// on a thread of its own, for the player whose life `player` watches, and
/// returns the way to its pictures.  The thread ends once it has decoded
/// the pass, or once the player has gone.
pub(super) fn start(path: PathBuf, clip: Clip, pass: u32, player: Weak<()>) -> Receiver<Sent> {
    let (sender, pictures) = mpsc::sync_channel(AHEAD);
    thread::spawn(move || {
        let end = match decoded(&path, clip, pass, &sender, &player) {
            Ok(0) => Sent::Failed(UNDECODABLE.to_owned()),
            Ok(_) => Sent::Ended,
            Err(Stop::Failed(reason)) => Sent::Failed(reason),
            Err(Stop::Gone) => return,
        };
        // Player that would hear of it may have gone since
        let _ = sender.send(end);
    });
    pictures
}

/// Decodes pass `pass` through `clip` of the file at `path`, and sends each
/// of its pictures to `sender` with the time of the run at which it is due,
/// for as long as the player whose life `player` watches is there.
/// Returns how many pictures it sent.
fn decoded(
    path: &Path,
    clip: Clip,
    pass: u32,
    sender: &SyncSender<Sent>,
    player: &Weak<()>,
) -> Result<usize, Stop> {
    let mut track = Track::open(path).map_err(Stop::Failed)?;
    let Some((from, to, first)) = track.span(clip.start, clip.end) else {
        return Ok(0);
    };
    let passed = f64::from(pass) * clip.length();
    // Pictures that show at their times, of which those that belong to the
    // clip go to the player, after the `sent` that went before them
    let send = |pictures: Vec<(f64, Decoded)>, sent: usize| {
        pictures
            .iter()
            .filter(|(shows, _)| (first..clip.end).contains(shows))
            .try_fold(sent, |sent, (shows, picture)| {
                let at = passed + (shows - clip.start).max(0.0);
                let frame = Frame {
                    at,
                    picture: rgba(picture),
                };
                sender
                    .send(Sent::Picture(frame))
                    .map(|()| sent + 1)
                    .map_err(|_| Stop::Gone)
            })
    };
    let mut decoder = Decoder::new();
    // Parameter sets that fail to decode leave every sample to fail as well
    track.parameter_sets().iter().for_each(|unit| {
        let _ = decoder.decode_nal(unit);
    });
    let order = Order::new(track.samples[from..=to].iter().map(|sample| sample.shows));
    let (_, sent) = (from..=to).try_fold((order, 0), |(order, sent), index| {
        if player.strong_count() == 0 {
            return Err(Stop::Gone);
        }
        let sample = track.sample(index).map_err(Stop::Failed)?;
        let picture = picture(&mut decoder, &track.units(&sample));
        let (due, order) = order.decoded(index - from, picture);
        Ok((order, send(due, sent)?))
    })?;
    Ok(sent)
}

/// Decodes the NAL units of a sample with `decoder`, and returns its
/// picture, once finished, when it decodes to one in eight bits for each
/// color with the colors shared by four pixels, which [`rgba`] turns into
/// RGBA
fn picture(decoder: &mut Decoder, units: &[NalUnit<'_>]) -> Option<Decoded> {
    // Unit that fails to decode adds nothing to the picture, which
    // decoding goes on without, and a sample that holds two pictures, as no
    // file should, keeps the second, which the decoder finishes last
    let earlier = units
        .iter()
        .filter_map(|unit| decoder.decode_nal(unit).ok().flatten())
        .last();
    decoder
        .flush()
        .or(earlier)
        .filter(|picture| picture.bit_depth == 8 && picture.chroma_format == ChromaFormat::Yuv420)
}
