// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Picture of a picture cue, which a thread of its own decodes from its
//! file, in PNG or JPEG
//!
//! The picture turns upright as its file tells, as a photograph taken on
//! its side does, and lies on black where it is not wholly opaque, as the
//! stage window shows it, since each picture there hides those under it.

use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    thread,
};

use image::{DynamicImage, ImageDecoder, ImageError, ImageReader};

use super::decoder::{Frame, Picture, Sent};
use crate::back::load::unreadable;

pub(super) const NO_PICTURE: &str = "holds no PNG or JPEG picture";

/// Reads the header of the picture file at `path`, without decoding its
/// picture, or tells why the picture cannot show
pub(in crate::back) fn probe(path: &Path) -> Result<(), String> {
    opened(path)?.into_dimensions().map(|_| ()).map_err(reason)
}

pub(super) fn start(path: PathBuf) -> Receiver<Sent> {
    let (sender, picture) = mpsc::channel();
    thread::spawn(move || {
        // Player that would take them may have gone since
        let _ = match decoded(&path) {
            Ok(picture) => sender
                .send(Sent::Picture(Frame { at: 0.0, picture }))
                .and_then(|()| sender.send(Sent::Ended)),
            Err(reason) => sender.send(Sent::Failed(reason)),
        };
    });
    picture
}

fn decoded(path: &Path) -> Result<Picture, String> {
    let mut decoder = opened(path)?.into_decoder().map_err(reason)?;
    let orientation = decoder.orientation().map_err(reason)?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(reason)?;
    image.apply_orientation(orientation);
    let opaque = !image.color().has_alpha();
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels = rgba.into_raw();
    Ok(Picture {
        width,
        height,
        pixels: if opaque { pixels } else { on_black(&pixels) },
    })
}

/// Opens the picture file at `path`, in the format that its first bytes
/// tell rather than the one that its name tells, or tells why it cannot be
/// read
fn opened(path: &Path) -> Result<ImageReader<BufReader<File>>, String> {
    File::open(path)
        .map(|file| ImageReader::new(BufReader::new(file)))
        .and_then(ImageReader::with_guessed_format)
        .map_err(unreadable)
}

fn on_black(pixels: &[u8]) -> Vec<u8> {
    let (pixels, _) = pixels.as_chunks::<4>();
    pixels
        .iter()
        .flat_map(|&[red, green, blue, alpha]| {
            let laid = |value: u8| (u16::from(value) * u16::from(alpha) / 255) as u8;
            [laid(red), laid(green), laid(blue), u8::MAX]
        })
        .collect()
}

fn reason(error: ImageError) -> String {
    match error {
        ImageError::Unsupported(_) => NO_PICTURE.to_owned(),
        ImageError::IoError(error) => unreadable(error),
        _ => error.to_string(),
    }
}
