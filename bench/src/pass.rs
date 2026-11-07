// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

//! Times a pass through a whole clip, as a thread of cinq's decodes it,
//! with a player that takes each picture as soon as it comes, so that the
//! pictures a second tell how fast a pass can go at most
//!
//! - `now`: cinq at ec61ead, which decodes each sample and converts its
//!   picture on the thread of the pass, high definition on scoped threads
//! - `pipelined`: the conversion of each picture on a thread of its own,
//!   with rayon's pool, while the thread of the pass decodes the next
//! - `threaded`: as `pipelined`, with rust_h264's `ThreadedDecoder`, which
//!   decodes several pictures at once, on one thread for each core
//!
//! Each with cinq's kernel of the conversion at ec61ead, `now`, or with
//! `pairs`, which the snapshot in the sandbox takes.

use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::Instant,
};

use cpu_time::ProcessTime;
use mp4::{MediaType, Mp4Reader};
use rust_h264::{
    decoder::{Decoder, Frame},
    nal::parse_avcc,
    threading::ThreadedDecoder,
};

use crate::color::{Kernel, Way, converted_with};

/// Pictures that the thread of a pass decodes ahead, as in cinq
const AHEAD: usize = 2;

#[derive(Debug, Clone, Copy)]
enum Arrangement {
    Now,
    Pipelined,
    Threaded,
}

#[derive(Debug, Clone, Copy)]
struct Variant(Arrangement, Kernel);

impl Variant {
    const ALL: [Self; 5] = [
        Self(Arrangement::Now, Kernel::Now),
        Self(Arrangement::Now, Kernel::Pairs),
        Self(Arrangement::Pipelined, Kernel::Pairs),
        Self(Arrangement::Threaded, Kernel::Now),
        Self(Arrangement::Threaded, Kernel::Pairs),
    ];

    fn name(self) -> String {
        let arrangement = match self.0 {
            Arrangement::Now => "now",
            Arrangement::Pipelined => "pipelined",
            Arrangement::Threaded => "threaded",
        };
        format!("{arrangement}, {}", self.1.name())
    }
}

/// Samples of the H.264 track of a clip, read into memory
struct Clip {
    parameters: Vec<u8>,
    length_size: usize,
    samples: Vec<Vec<u8>>,
}

fn read(path: &Path) -> Clip {
    let file = File::open(path).expect("open the clip");
    let size = file.metadata().expect("size of the clip").len();
    let mut reader = Mp4Reader::read_header(BufReader::new(file), size).expect("MP4 header");
    let track = reader
        .tracks()
        .values()
        .find(|track| track.media_type().is_ok_and(|kind| kind == MediaType::H264))
        .expect("H.264 track");
    let id = track.track_id();
    let count = track.sample_count();
    let avc = &track.trak.mdia.minf.stbl.stsd.avc1.as_ref().expect("avc1").avcc;
    let length_size = usize::from(avc.length_size_minus_one) + 1;
    let parameters = avc
        .sequence_parameter_sets
        .iter()
        .chain(&avc.picture_parameter_sets)
        .flat_map(|set| {
            (set.bytes.len() as u32)
                .to_be_bytes()
                .into_iter()
                .chain(set.bytes.iter().copied())
        })
        .collect();
    let samples = (1..=count)
        .map(|number| {
            reader
                .read_sample(id, number)
                .expect("sample")
                .map(|sample| sample.bytes.to_vec())
                .unwrap_or_default()
        })
        .collect();
    Clip {
        parameters,
        length_size,
        samples,
    }
}

pub fn run(paths: &[String]) {
    let cores = thread::available_parallelism().map_or(1, |cores| cores.get());
    println!("## Passes through whole clips, {cores} cores");
    println!();
    println!("| clip | variant | pictures | pictures a second | cores busy |");
    println!("| --- | --- | --- | --- | --- |");
    paths.iter().for_each(|path| {
        let path = Path::new(path);
        let name = path.file_stem().unwrap_or_default().to_string_lossy();
        let clip = Arc::new(read(path));
        // Warms the file cache, the allocator, and rayon's pool
        let _ = pass(&clip, Variant(Arrangement::Pipelined, Kernel::Now));
        // Best of three, or of BENCH_REPEAT, as other jobs on the host come
        // and go
        let repeat = std::env::var("BENCH_REPEAT")
            .ok()
            .and_then(|repeat| repeat.parse().ok())
            .unwrap_or(3usize);
        Variant::ALL.iter().for_each(|&variant| {
            let (count, seconds, cpu) = (0..repeat.max(1))
                .map(|_| pass(&clip, variant))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .expect("a pass");
            println!(
                "| {name} | {} | {count} | {:.1} | {:.2} |",
                variant.name(),
                count as f64 / seconds,
                cpu / seconds,
            );
        });
    });
    println!();
}

/// Returns how many pictures a pass sent, and the wall and processor
/// seconds that it took
fn pass(clip: &Arc<Clip>, variant: Variant) -> (usize, f64, f64) {
    let start = Instant::now();
    let cpu = ProcessTime::now();
    let pictures = match variant {
        Variant(Arrangement::Now, kernel) => {
            decoding(clip, move |frame| converted_with(&frame, now_way(&frame), kernel))
        }
        Variant(Arrangement::Pipelined, kernel) => converting(decoding(clip, |frame| frame), kernel),
        Variant(Arrangement::Threaded, kernel) => converting(threaded(clip), kernel),
    };
    let count = pictures.iter().count();
    (
        count,
        start.elapsed().as_secs_f64(),
        cpu.elapsed().as_secs_f64(),
    )
}

/// Way that cinq at ec61ead converts a picture: on threads from 1280 wide
/// or 720 lines up
fn now_way(frame: &Frame) -> Way {
    if frame.width >= 1280 || frame.height >= 720 {
        Way::Scoped
    } else {
        Way::Serial
    }
}

/// Decodes the clip on a thread of its own, sample by sample as cinq does,
/// and sends what `then` makes of each picture
fn decoding<T: Send + 'static>(
    clip: &Arc<Clip>,
    then: impl Fn(Frame) -> T + Send + 'static,
) -> Receiver<T> {
    let (sender, receiver) = mpsc::sync_channel(AHEAD);
    let clip = Arc::clone(clip);
    thread::spawn(move || {
        let mut decoder = Decoder::new();
        parse_avcc(&clip.parameters, 4).iter().for_each(|unit| {
            let _ = decoder.decode_nal(unit);
        });
        let _ = clip.samples.iter().try_for_each(|sample| {
            let units = parse_avcc(sample, clip.length_size);
            let earlier = units
                .iter()
                .filter_map(|unit| decoder.decode_nal(unit).ok().flatten())
                .last();
            match decoder.flush().or(earlier) {
                Some(frame) => sender.send(then(frame)),
                None => Ok(()),
            }
        });
    });
    receiver
}

/// Decodes the clip with rust_h264's threaded decoder on a thread of its
/// own, which sends its pictures in the order they show
fn threaded(clip: &Arc<Clip>) -> Receiver<Frame> {
    let (sender, receiver) = mpsc::sync_channel(AHEAD);
    let clip = Arc::clone(clip);
    thread::spawn(move || {
        let threads = thread::available_parallelism().map_or(1, |cores| cores.get());
        let mut decoder = ThreadedDecoder::new(threads);
        parse_avcc(&clip.parameters, 4).iter().for_each(|unit| {
            let _ = decoder.decode_nal(unit);
        });
        let send = |sender: &SyncSender<Frame>, frames: Vec<Frame>| {
            frames.into_iter().try_for_each(|frame| sender.send(frame))
        };
        let sent = clip.samples.iter().try_for_each(|sample| {
            parse_avcc(sample, clip.length_size)
                .iter()
                .try_for_each(|unit| send(&sender, decoder.decode_nal(unit).unwrap_or_default()))
        });
        if sent.is_ok() {
            let _ = send(&sender, decoder.flush());
        }
    });
    receiver
}

/// Converts the pictures of `frames` on a thread of its own, with rayon's
/// pool, while the pictures after them decode
fn converting(frames: Receiver<Frame>, kernel: Kernel) -> Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::sync_channel(AHEAD);
    thread::spawn(move || {
        let _ = frames
            .into_iter()
            .try_for_each(|frame| sender.send(converted_with(&frame, Way::RayonStrips, kernel)));
    });
    receiver
}
