// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of mixing sounds into the channels of a device, which need no
//! device: the mix renders into a buffer as the stream of a device would

use std::{sync::Arc, time::Duration};

use kira::{
    AudioManager, AudioManagerSettings, Frame,
    backend::cpal::cpal::{ErrorKind, I24, SampleFormat},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use rtrb::{Consumer, Producer, RingBuffer};

use super::mix::{Command, Gains, Gone, Mix};
use crate::back::audio::feed::Feed;

const RATE: u32 = 48000;

/// Frames of the ten milliseconds that a sound takes to fade out
const RAMP: usize = RATE as usize / 100;

/// Frames of each render, as a device asks for them
const FRAMES: usize = 256;

/// Left and right of the sound that the tests play, apart so that the
/// tests can tell where each went
const SOUND: Frame = Frame {
    left: 0.5,
    right: -0.25,
};

struct Bench {
    mix: Mix,
    rate: u32,
    channels: usize,
    commands: Producer<Command>,
    gone: Consumer<Gone>,
    /// Managers of the sounds, which hand the sounds their commands
    _managers: Vec<AudioManager<Feed>>,
}

impl Bench {
    fn new(channels: usize) -> Self {
        Self::at(RATE, channels)
    }

    fn at(rate: u32, channels: usize) -> Self {
        let (commands, taken) = RingBuffer::new(16);
        let (handed, gone) = RingBuffer::new(16);
        Self {
            mix: Mix::new(channels, rate, 8, taken, handed),
            rate,
            channels,
            commands,
            gone,
            _managers: Vec::new(),
        }
    }

    /// Plays a second of [`SOUND`] at `gains`
    fn play(mut self, id: u64, gains: Gains) -> Self {
        let rate = self.rate;
        let Ok(mut manager) = AudioManager::<Feed>::new(AudioManagerSettings {
            backend_settings: rate,
            ..AudioManagerSettings::default()
        });
        manager
            .play(StaticSoundData {
                sample_rate: rate,
                frames: Arc::from(vec![SOUND; rate as usize]),
                settings: StaticSoundSettings::default(),
                slice: None,
            })
            .expect("a sound plays");
        let renderer = manager
            .backend_mut()
            .take()
            .expect("kira starts the backend");
        let sent = self.commands.push(Command::add(id, renderer, gains));
        assert!(sent.is_ok(), "the ring takes the sound");
        self._managers.push(manager);
        self
    }

    fn remove(mut self, id: u64) -> Self {
        let sent = self.commands.push(Command::Remove(id));
        assert!(sent.is_ok(), "the ring takes the command");
        self
    }

    /// Renders [`FRAMES`] frames and returns each channel of them
    fn render(&mut self) -> Vec<Vec<f32>> {
        let mut out = vec![1.0; FRAMES * self.channels];
        self.mix.render(&mut out);
        (0..self.channels)
            .map(|channel| {
                out.iter()
                    .skip(channel)
                    .step_by(self.channels)
                    .copied()
                    .collect()
            })
            .collect()
    }
}

/// Gains of a device of `channels` channels, which send left and right
/// as `taps` lists them for each channel, and nothing to the others
fn gains(channels: usize, taps: &[(usize, [f32; 2])]) -> Gains {
    (0..channels)
        .map(|channel| {
            taps.iter()
                .find(|(tap, _)| *tap == channel)
                .map_or([0.0; 2], |(_, gain)| *gain)
        })
        .collect()
}

/// Returns whether every sample of `channel` past the first few, which
/// kira's interpolation leans into from before the start, is `value`
fn steady(channel: &[f32], value: f32) -> bool {
    channel[8..]
        .iter()
        .all(|sample| (sample - value).abs() < 1e-4)
}

#[test]
fn sound_plays_on_the_channels_that_its_gains_name_and_nowhere_else() {
    let mut bench = Bench::new(4).play(1, gains(4, &[(2, [1.0, 0.0]), (3, [0.0, 1.0])]));
    let channels = bench.render();
    assert!(
        channels[0]
            .iter()
            .chain(&channels[1])
            .all(|sample| *sample == 0.0)
    );
    assert!(steady(&channels[2], SOUND.left), "{:?}", &channels[2][..12]);
    assert!(
        steady(&channels[3], SOUND.right),
        "{:?}",
        &channels[3][..12]
    );
}

#[test]
fn sounds_on_one_channel_add_up_and_a_mono_channel_takes_both_halves() {
    let mut bench = Bench::new(2)
        .play(1, gains(2, &[(0, [1.0, 0.0])]))
        .play(2, gains(2, &[(0, [1.0, 0.0]), (1, [0.5, 0.5])]));
    let channels = bench.render();
    assert!(steady(&channels[0], 2.0 * SOUND.left));
    assert!(steady(&channels[1], (SOUND.left + SOUND.right) / 2.0));
}

#[test]
fn sound_that_goes_fades_out_and_comes_back_to_be_freed() {
    let mut bench = Bench::new(2).play(1, gains(2, &[(0, [1.0, 0.0])]));
    bench.render();
    let mut bench = bench.remove(1);
    let fading: Vec<f32> = (0..RAMP.div_ceil(FRAMES))
        .flat_map(|_| bench.render().swap_remove(0))
        .collect();
    assert!(fading[..RAMP].windows(2).all(|pair| pair[1] <= pair[0]));
    assert!(fading[0] > 0.9 * SOUND.left && fading[RAMP - 1].abs() < 1e-6);
    assert!(bench.gone.pop().is_ok(), "the sound comes back to be freed");
    assert!(bench.render()[0].iter().all(|sample| *sample == 0.0));
}

#[test]
fn stream_that_has_yet_to_call_has_longer_than_one_that_has() {
    let seconds = Duration::from_secs;
    assert!(super::stalled(seconds(3), Some(Duration::ZERO)));
    assert!(!super::stalled(seconds(3), None));
    assert!(super::stalled(seconds(10), None));
    assert!(!super::stalled(seconds(11), Some(seconds(9))));
}

#[test]
fn fade_out_lasts_ten_milliseconds_at_any_rate() {
    let ramp: usize = 96000 / 100;
    let mut bench = Bench::at(96000, 2).play(1, gains(2, &[(0, [1.0, 0.0])]));
    bench.render();
    let mut bench = bench.remove(1);
    let fading: Vec<f32> = (0..ramp.div_ceil(FRAMES))
        .flat_map(|_| bench.render().swap_remove(0))
        .collect();
    assert!(fading[ramp - 2] > 1e-4 && fading[ramp - 1].abs() < 1e-6);
}

#[test]
fn sum_past_full_scale_clips_instead_of_wrapping() {
    let mut bench = Bench::new(2).play(1, gains(2, &[(0, [3.0, 0.0])]));
    let mut wide = vec![0.0_f32; FRAMES * 2];
    bench.mix.render(&mut wide);
    assert!(
        wide[16..]
            .chunks(2)
            .all(|frame| frame[0] > 0.999 && frame[0] < 1.0)
    );
    let mut narrow = vec![0_i16; FRAMES * 2];
    bench.mix.render(&mut narrow);
    assert!(narrow[16..].chunks(2).all(|frame| frame[0] == i16::MAX));
}

#[test]
fn sum_past_full_scale_stops_at_the_last_value_of_24_bits() {
    let mut bench = Bench::new(2).play(1, gains(2, &[(0, [3.0, 0.0])]));
    let mut out = vec![I24::new_unchecked(0); FRAMES * 2];
    bench.mix.render(&mut out);
    assert!(
        out[16..]
            .chunks(2)
            .all(|frame| frame[0].inner() == 8_388_607)
    );
}

#[test]
fn integer_samples_take_the_sound_at_its_level() {
    let mut bench = Bench::new(2).play(1, gains(2, &[(0, [1.0, 0.0]), (1, [0.0, 1.0])]));
    let mut wide = vec![0_i32; FRAMES * 2];
    bench.mix.render(&mut wide);
    let mut narrow = vec![0_i16; FRAMES * 2];
    bench.mix.render(&mut narrow);
    let near = |sample: f64, value: f32, full: f64| (sample / full - f64::from(value)).abs() < 1e-3;
    assert!(wide[16..].chunks(2).all(|frame| {
        near(f64::from(frame[0]), SOUND.left, 2_f64.powi(31))
            && near(f64::from(frame[1]), SOUND.right, 2_f64.powi(31))
    }));
    assert!(narrow.chunks(2).all(|frame| {
        near(f64::from(frame[0]), SOUND.left, 2_f64.powi(15))
            && near(f64::from(frame[1]), SOUND.right, 2_f64.powi(15))
    }));
}

#[test]
fn stream_is_built_anew_after_its_device_goes_but_not_after_an_underrun() {
    assert!(super::stream::ends(ErrorKind::DeviceNotAvailable));
    assert!(super::stream::ends(ErrorKind::StreamInvalidated));
    assert!(!super::stream::ends(ErrorKind::Xrun));
    assert!(!super::stream::ends(ErrorKind::BackendError));
}

#[test]
fn device_opens_with_the_fewest_channels_that_hold_the_outputs() {
    let offered = [
        (SampleFormat::F32, 2),
        (SampleFormat::F32, 16),
        (SampleFormat::I32, 64),
    ];
    assert_eq!(
        super::config::chosen(&offered, 3),
        Some((SampleFormat::F32, 16))
    );
    assert_eq!(
        super::config::chosen(&offered, 2),
        Some((SampleFormat::F32, 2))
    );
}

#[test]
fn device_of_too_few_channels_opens_with_all_it_has() {
    let offered = [(SampleFormat::F32, 2), (SampleFormat::F32, 16)];
    assert_eq!(
        super::config::chosen(&offered, 34),
        Some((SampleFormat::F32, 16))
    );
}

#[test]
fn device_of_too_few_channels_opens_with_the_most_of_any_samples() {
    let offered = [(SampleFormat::F32, 2), (SampleFormat::I32, 8)];
    assert_eq!(
        super::config::chosen(&offered, 34),
        Some((SampleFormat::I32, 8))
    );
}

#[test]
fn channels_that_fit_come_before_the_samples_that_kira_renders() {
    let offered = [(SampleFormat::F32, 2), (SampleFormat::I16, 8)];
    assert_eq!(
        super::config::chosen(&offered, 4),
        Some((SampleFormat::I16, 8))
    );
}

#[test]
fn device_of_no_samples_that_cinq_plays_does_not_open() {
    assert_eq!(super::config::chosen(&[], 2), None);
    assert_eq!(super::config::chosen(&[(SampleFormat::U8, 2)], 2), None);
}

#[test]
fn hw_device_of_samples_that_cinq_cannot_play_names_its_plughw_device() {
    assert_eq!(
        super::config::unplayable("alsa:hw:CARD=USB,DEV=0"),
        "it takes no samples that cinq can play; choose alsa:plughw:CARD=USB,DEV=0 instead"
    );
    assert_eq!(
        super::config::unplayable("asio:FlexASIO"),
        "it takes no samples that cinq can play"
    );
}
