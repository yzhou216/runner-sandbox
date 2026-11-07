// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

//! Decodes the H.264 track of an MP4 file with rust_h264, as cinq does, and
//! with openh264, as cinq did, checks every picture of each against the
//! MD5 that FFmpeg's framemd5 gives it, and times both decoders.
//!
//! Usage: compare-h264 FILE.mp4 FILE.framemd5

use std::{
    env,
    fs::{self, File},
    io::BufReader,
    time::{Duration, Instant},
};

use mp4::{MediaType, Mp4Reader};
use openh264::{
    OpenH264API,
    decoder::{Decoder as OpenDecoder, DecoderConfig, Flush},
    formats::YUVSource,
};
use rust_h264::{decoder::Decoder as RustDecoder, nal::parse_avcc};

/// Sample of the track, in the order of decoding, with the time it shows
struct Sample {
    bytes: Vec<u8>,
    shows: i64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (path, sums) = (&args[1], &args[2]);
    let expected: Vec<String> = fs::read_to_string(sums)
        .expect("the framemd5 file reads")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.rsplit(',').next().unwrap().trim().to_owned())
        .collect();
    let file = File::open(path).expect("the video opens");
    let size = file.metadata().unwrap().len();
    let mut reader = Mp4Reader::read_header(BufReader::new(file), size).expect("MP4 reads");
    let track = reader
        .tracks()
        .values()
        .find(|track| track.media_type().is_ok_and(|kind| kind == MediaType::H264))
        .expect("an H.264 track");
    let id = track.track_id();
    let count = track.sample_count();
    let (width, height) = (track.width(), track.height());
    let avc = &track.trak.mdia.minf.stbl.stsd.avc1.as_ref().unwrap().avcc;
    let length_size = usize::from(avc.length_size_minus_one) + 1;
    let sets: Vec<Vec<u8>> = avc
        .sequence_parameter_sets
        .iter()
        .chain(&avc.picture_parameter_sets)
        .map(|set| set.bytes.clone())
        .collect();
    let samples: Vec<Sample> = (1..=count)
        .map(|number| {
            let sample = reader.read_sample(id, number).unwrap().unwrap();
            Sample {
                bytes: sample.bytes.to_vec(),
                shows: sample.start_time as i64 + i64::from(sample.rendering_offset),
            }
        })
        .collect();

    let (rust_sums, rust_time) = rust(&sets, &samples, length_size);
    let (open_sums, open_time) = open(&sets, &samples, length_size);
    let (threaded_sums, threaded_time) = threaded(&sets, &samples, length_size, 4);
    let matching = |sums: &[String]| sums.iter().zip(&expected).filter(|(a, b)| a == b).count();
    let fps = |frames: usize, time: Duration| frames as f64 / time.as_secs_f64();
    println!(
        "{path}: {width}x{height}, {count} samples, FFmpeg {} pictures",
        expected.len()
    );
    println!(
        "  rust_h264 {:4} pictures, {:4} match FFmpeg, {:7.1} pictures/s",
        rust_sums.len(),
        matching(&rust_sums),
        fps(rust_sums.len(), rust_time)
    );
    println!(
        "  openh264  {:4} pictures, {:4} match FFmpeg, {:7.1} pictures/s",
        open_sums.len(),
        matching(&open_sums),
        fps(open_sums.len(), open_time)
    );
    println!(
        "  rust_h264 on 4 threads {:4} pictures, {:4} match FFmpeg, {:7.1} pictures/s",
        threaded_sums.len(),
        matching(&threaded_sums),
        fps(threaded_sums.len(), threaded_time)
    );
    let anywhere = |sums: &[String]| sums.iter().filter(|sum| expected.contains(sum)).count();
    println!(
        "  found anywhere in FFmpeg's pictures: rust_h264 {}, openh264 {}",
        anywhere(&rust_sums),
        anywhere(&open_sums)
    );
    if let Ok(dump) = env::var("DUMP_OPENH264") {
        fs::write(&dump, open_frames(&sets, &samples, length_size)).expect("the dump writes");
    }
    let mismatched: Vec<usize> = rust_sums
        .iter()
        .zip(&expected)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(index, _)| index)
        .take(10)
        .collect();
    if !mismatched.is_empty() {
        println!("  rust_h264 differs from FFmpeg at pictures {mismatched:?}");
    }
}

/// Decodes the samples with rust_h264 as cinq does: each sample finished
/// once read, pictures in the order of the times they show
fn rust(sets: &[Vec<u8>], samples: &[Sample], length_size: usize) -> (Vec<String>, Duration) {
    let parameters: Vec<u8> = sets
        .iter()
        .flat_map(|set| (set.len() as u32).to_be_bytes().into_iter().chain(set.iter().copied()))
        .collect();
    let mut decoder = RustDecoder::new();
    parse_avcc(&parameters, 4).iter().for_each(|unit| {
        let _ = decoder.decode_nal(unit);
    });
    let mut time = Duration::ZERO;
    let mut pictures: Vec<(i64, String)> = samples
        .iter()
        .filter_map(|sample| {
            let units = parse_avcc(&sample.bytes, length_size);
            let start = Instant::now();
            let first = units
                .iter()
                .filter_map(|unit| decoder.decode_nal(unit).ok().flatten())
                .last();
            let picture = decoder.flush().or(first);
            time += start.elapsed();
            picture.map(|picture| {
                let digest = md5::compute([picture.y, picture.u, picture.v].concat());
                (sample.shows, format!("{digest:x}"))
            })
        })
        .collect();
    pictures.sort_by_key(|(shows, _)| *shows);
    (pictures.into_iter().map(|(_, sum)| sum).collect(), time)
}

/// Decodes the samples with openh264 as cinq did, flushing at the end
fn open(sets: &[Vec<u8>], samples: &[Sample], length_size: usize) -> (Vec<String>, Duration) {
    let annex_b = |units: Vec<&[u8]>| -> Vec<u8> {
        units
            .into_iter()
            .flat_map(|unit| [0, 0, 0, 1].into_iter().chain(unit.iter().copied()))
            .collect()
    };
    let units = |bytes: &[u8]| -> Vec<Vec<u8>> {
        let mut units = Vec::new();
        let mut rest = bytes;
        while rest.len() >= length_size {
            let length = rest[..length_size]
                .iter()
                .fold(0_usize, |length, &byte| length << 8 | usize::from(byte));
            let Some(unit) = rest.get(length_size..length_size + length) else {
                break;
            };
            units.push(unit.to_vec());
            rest = &rest[length_size + length..];
        }
        units
    };
    let config = DecoderConfig::new().flush_after_decode(Flush::NoFlush);
    let mut decoder =
        OpenDecoder::with_api_config(OpenH264API::from_source(), config).expect("openh264 starts");
    let mut time = Duration::ZERO;
    let mut sums = Vec::new();
    for (index, sample) in samples.iter().enumerate() {
        let sample_units = units(&sample.bytes);
        let all: Vec<&[u8]> = if index == 0 {
            sets.iter().map(Vec::as_slice).chain(sample_units.iter().map(Vec::as_slice)).collect()
        } else {
            sample_units.iter().map(Vec::as_slice).collect()
        };
        let packet = annex_b(all);
        let start = Instant::now();
        let decoded = decoder.decode(&packet);
        time += start.elapsed();
        if let Ok(Some(picture)) = decoded {
            sums.push(digest(&picture));
        }
    }
    let start = Instant::now();
    let held = decoder.flush_remaining().unwrap_or_default();
    time += start.elapsed();
    sums.extend(held.iter().map(digest));
    (sums, time)
}

/// Returns the MD5 of a picture of openh264 laid out as FFmpeg's yuv420p
fn digest(picture: &impl YUVSource) -> String {
    let (width, height) = picture.dimensions();
    let (luma, blue, red) = picture.strides();
    let rows = |plane: &[u8], stride: usize, width: usize, height: usize| -> Vec<u8> {
        (0..height)
            .flat_map(|row| plane[row * stride..][..width].iter().copied())
            .collect()
    };
    let bytes = [
        rows(picture.y(), luma, width, height),
        rows(picture.u(), blue, width / 2, height / 2),
        rows(picture.v(), red, width / 2, height / 2),
    ]
    .concat();
    format!("{:x}", md5::compute(bytes))
}

/// Returns every picture that openh264 decodes, in the order it hands them
/// out, laid out as FFmpeg's yuv420p, one after another
fn open_frames(sets: &[Vec<u8>], samples: &[Sample], length_size: usize) -> Vec<u8> {
    let config = DecoderConfig::new().flush_after_decode(Flush::NoFlush);
    let mut decoder =
        OpenDecoder::with_api_config(OpenH264API::from_source(), config).expect("openh264 starts");
    let planes = |picture: &openh264::decoder::DecodedYUV<'_>| -> Vec<u8> {
        let (width, height) = picture.dimensions();
        let (luma, blue, red) = picture.strides();
        let rows = |plane: &[u8], stride: usize, width: usize, height: usize| -> Vec<u8> {
            (0..height).flat_map(|row| plane[row * stride..][..width].iter().copied()).collect()
        };
        [
            rows(picture.y(), luma, width, height),
            rows(picture.u(), blue, width / 2, height / 2),
            rows(picture.v(), red, width / 2, height / 2),
        ]
        .concat()
    };
    let mut out = Vec::new();
    for (index, sample) in samples.iter().enumerate() {
        let mut packet = Vec::new();
        if index == 0 {
            sets.iter().for_each(|set| packet.extend([0, 0, 0, 1].iter().chain(set)));
        }
        let mut rest = &sample.bytes[..];
        while rest.len() >= length_size {
            let length = rest[..length_size].iter().fold(0_usize, |l, &b| l << 8 | usize::from(b));
            let Some(unit) = rest.get(length_size..length_size + length) else { break };
            packet.extend([0, 0, 0, 1].iter().chain(unit));
            rest = &rest[length_size + length..];
        }
        if let Ok(Some(picture)) = decoder.decode(&packet) {
            out.extend(planes(&picture));
        }
    }
    decoder.flush_remaining().unwrap_or_default().iter().for_each(|picture| out.extend(planes(picture)));
    out
}

/// Decodes the samples with rust_h264's frame-threaded decoder on
/// `threads` threads, which hands out pictures in the order they show
pub fn threaded(sets: &[Vec<u8>], samples: &[Sample], length_size: usize, threads: usize) -> (Vec<String>, Duration) {
    let parameters: Vec<u8> = sets
        .iter()
        .flat_map(|set| (set.len() as u32).to_be_bytes().into_iter().chain(set.iter().copied()))
        .collect();
    let mut decoder = rust_h264::threading::ThreadedDecoder::new(threads);
    let mut frames = Vec::new();
    let start = Instant::now();
    parse_avcc(&parameters, 4).iter().for_each(|unit| {
        let _ = decoder.decode_nal(unit);
    });
    for sample in samples {
        for unit in parse_avcc(&sample.bytes, length_size) {
            if let Ok(out) = decoder.decode_nal(&unit) {
                frames.extend(out);
            }
        }
    }
    frames.extend(decoder.flush());
    let time = start.elapsed();
    let sums = frames
        .into_iter()
        .map(|frame| format!("{:x}", md5::compute([frame.y, frame.u, frame.v].concat())))
        .collect();
    (sums, time)
}
