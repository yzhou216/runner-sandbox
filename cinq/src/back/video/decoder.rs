// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Threads that decode the clip of one run of a video cue, a pass through
//! the clip each, a few pictures ahead of the clock of the run
//!
//! A pass starts a decoder at the last sync sample before the start of the
//! clip that shows no later than the first picture of the clip, and decodes
//! from there to the last sample that shows before its end.  The pictures
//! that the decoder hands out take the times that [`times`] keeps, and
//! those before the start of the clip are let go without turning them into
//! RGBA, which [`color`] turns the rest into.  The decoder flushes only
//! where a closed group of pictures starts, and once the pass has read
//! every sample, since flushing after each sample, as openh264 does by
//! default, fails on the second picture of streams with B-frames.  A pass
//! ends by telling its player that it has sent every picture, or why its
//! picture cannot play, as a pass that decodes no picture cannot, and it
//! stops decoding as soon as its player has gone.  Each pass opens its file
//! and reads the index of its track anew, which takes a long file a moment,
//! hidden for a clip that loops by the pass that decodes ahead.

mod color;
mod times;

use std::{
    path::{Path, PathBuf},
    sync::{
        Weak,
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
};

use openh264::{
    OpenH264API,
    decoder::{DecodedYUV, Decoder, DecoderConfig, Flush},
};

use super::track::Track;
use crate::back::Clip;
pub use color::Picture;
use color::rgba;
use times::Times;

/// Pictures that the thread of a pass decodes ahead of the clock of its
/// run, besides the one it holds as it waits to send it: enough to ride out
/// the pictures that take longest to decode, such as those that start a
/// group of pictures, and few enough to keep the memory of a pass small,
/// as each picture of 1080p takes 8 MB
const AHEAD: usize = 2;

/// Why the picture of a file that holds H.264 cannot play, when openh264
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
    /// Picture cannot play, for the given reason
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
    let times = Times::new(track.samples[from..=to].iter().map(|sample| sample.shows));
    let passed = f64::from(pass) * (clip.end - clip.start);
    // Picture that the decoder hands out, which shows at `shows`, and goes
    // to the player when it belongs to the clip
    let send = |picture: DecodedYUV<'_>, shows: Option<f64>| match shows {
        Some(shows) if (first..clip.end).contains(&shows) => {
            let at = passed + (shows - clip.start).max(0.0);
            let frame = Frame {
                at,
                picture: rgba(&picture),
            };
            sender
                .send(Sent::Picture(frame))
                .map(|()| 1)
                .map_err(|_| Stop::Gone)
        }
        _ => Ok(0),
    };
    let config = DecoderConfig::new().flush_after_decode(Flush::NoFlush);
    let mut decoder = Decoder::with_api_config(OpenH264API::from_source(), config)
        .map_err(|error| Stop::Failed(error.to_string()))?;
    let (times, sent) = (from..=to).try_fold((times, 0), |(times, sent), index| {
        if player.strong_count() == 0 {
            return Err(Stop::Gone);
        }
        // Closed group of pictures that starts here owes nothing to the
        // pictures before it, so the decoder hands those out first, and
        // the times start again from the first picture of the group
        let (times, sent) = if index > from && track.starts_group(index) {
            let held = decoder.flush_remaining().unwrap_or_default();
            let (times, flushed) = handed_out(held, times, &send)?;
            (times.from(track.samples[index].shows), sent + flushed)
        } else {
            (times, sent)
        };
        let packet = track.packet(index, index == from).map_err(Stop::Failed)?;
        // Sample that fails to decode gives no picture, and decoding goes on
        match decoder.decode(&packet) {
            Ok(Some(picture)) => {
                let (shows, times) = times.next();
                Ok((times, sent + send(picture, shows)?))
            }
            _ => Ok((times, sent)),
        }
    })?;
    let held = decoder.flush_remaining().unwrap_or_default();
    let (_, flushed) = handed_out(held, times, &send)?;
    Ok(sent + flushed)
}

/// Sends the pictures that the decoder hands out at once with `send`, each
/// with the next of `times`.  Returns the times left, along with how many
/// pictures went to the player.
fn handed_out(
    pictures: Vec<DecodedYUV<'_>>,
    times: Times,
    send: &impl Fn(DecodedYUV<'_>, Option<f64>) -> Result<usize, Stop>,
) -> Result<(Times, usize), Stop> {
    pictures
        .into_iter()
        .try_fold((times, 0), |(times, sent), picture| {
            let (shows, times) = times.next();
            Ok((times, sent + send(picture, shows)?))
        })
}
