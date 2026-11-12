<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: outputs and channels

This branch records cinq routing one audio cue to the outputs of a show
that share one device of eight channels, from its main window alone: the
outputs view in place of the cue list, the sends of the cue on its Levels
tab, and the channels of an output changed while the cue plays.  `cinq/`
holds the tree of cinq that was tested, from the `audio-outputs` branch of
yzhou216/cinq.

## What runs

`.github/workflows/outputs.yml` builds cinq with `nix build ./cinq`, makes
the sound of `desk.cinq.toml` with `media.sh`, and runs `demo.sh`,
`measure.sh`, and `compose.sh`.  The artifact `recording` holds what they
write.

- `demo.sh` starts a PulseAudio server with one null sink of eight
  channels, `desk`, which `~/.asoundrc` names as the ALSA device
  `alsa:desk` ("Desk").  The show has four outputs there: House on 1-2,
  Sub on 3, Monitors on 5-6, and Lobby on 7-8.  Lobby makes cinq open all
  eight channels, since PulseAudio spreads a stream of fewer channels than
  its sink over every channel of the sink.
- A headless sway session holds the main window, 1280x900.  `wf-recorder`
  records it, and `parecord` records the monitor of the sink with all
  eight channels.  `wtype` types the keys of the vi keymap.
- The script steps to the outputs with l, selects Sub with j, goes back
  with h, and plays the cue with Space; it sends to House alone, as new
  cues do.  On the Levels tab, it types 0 for Sub and -6 for Monitors and
  empties House.  It then shows the outputs again and changes the channels
  of Sub from 3 to 4 while the cue plays.  Escape is Panic, and Control+s
  saves the show, which the script keeps as `saved.cinq.toml`.
- `measure.sh` prints the RMS level of each channel between each step and
  the next, lining up the recording by the onset of the music after GO.
- `compose.sh` puts the screen beside the waveform of each channel and the
  captions below.  Odd channels play in the left ear and even ones in the
  right, so that Sub moving from channel 3 to channel 4 moves between
  them.

## Results

Run 43199066899 on `ubuntu-latest` passed, testing the tree in the first
commit of this branch, an earlier state of audio-outputs.  Run 43243606327
passed on the same label, testing the tree of commit f27f2fe, which
`cinq/` now holds.  Each took about 16 minutes, 13 of them spent
building cinq and the tools of the session.

Both runs gave the same levels, as RMS levels in dBFS, with `-` for
silence:

| Step                 | 1     | 2     | 3     | 4     | 5     | 6     | 7 | 8 |
| -------------------- | ----- | ----- | ----- | ----- | ----- | ----- | - | - |
| GO, to House alone   | -16.5 | -16.5 | -     | -     | -     | -     | - | - |
| Sub at 0 dB          | -16.5 | -16.5 | -16.5 | -     | -     | -     | - | - |
| Monitors at -6 dB    | -16.5 | -16.5 | -16.5 | -     | -22.5 | -22.5 | - | - |
| House emptied        | -     | -     | -16.5 | -     | -22.5 | -22.5 | - | - |
| Sub on channel 4     | -     | -     | -     | -16.5 | -22.5 | -22.5 | - | - |
| Panic                | -     | -     | -     | -     | -     | -     | - | - |

- Each step reached the channels that it names and no others, and the
  Monitors played 6.0 dB below the rest.  A send of 0 dB plays the
  sound at the level of its file, -16.5 dBFS on each channel.
  `saved.cinq.toml` held Sub on channel 4 and the sends of the cue,
  Sub at 0 and Monitors at -6.
- In run 43199066899, every output of the desk showed "ALSA function
  'snd_pcm_avail_delay' failed with error 'I/O error (5)'" among the
  warnings from midway on, while the sound played on as measured.
  cpal's ALSA worker plays on after such an error, where its WASAPI
  worker ends the stream after any failed call, so the kind of an error
  cannot tell whether the stream stopped.  cinq now counts a stream as
  stopped once it has asked for no sound for three seconds.  In run
  43243606327, every step showed no warnings.  On the local machine,
  unloading the sink while the cue played showed the warning on each
  output within 5.5 seconds.
- On the local machine, a show with outputs on channels 1 to 3 alone of
  the same sink opened a stream of three channels, front-left,
  front-right, and aux0, and PulseAudio played it on every channel of
  the sink but the LFE.  The show here gives the desk an output on 7-8
  so that cinq opens all eight.  This is open in cinq.
- ffmpeg 9's showwaves drew the colors `0x7fd7ff` and `0xffb070` as
  crimson, while named colors work, and `draw=full` keeps the lines
  bright.  parecord warns that it cannot write the channel map to the
  WAV file, which keeps the channels in the order of the stream.
