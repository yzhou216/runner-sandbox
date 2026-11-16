<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's audio routing, with the keys of its outputs view

Tests the audio routing of cinq on ubuntu-latest, windows-latest, and
macos-latest.  `cinq/` holds the snapshot of commit 6bddf30 of the `audio-fix`
branch of yzhou216/cinq (dated 2026-11-15 15:29:42 -0800), with two edits for
this branch alone: a patch that prints each key that no widget took (`K`), the
message and the state after each update (`U`), and the start and end of each
opening of a device (`O`), with the time in seconds since the epoch, to
standard error; and `doCheck = false` in `flake/cinq.nix`, since the test
`device_keeps_its_room_while_the_other_columns_give_way` measures the system's
fonts and failed in a local Nix build.  `verdict.py` judges each check from
those lines and from the recordings, and the screenshots only confirm it.

No Space ever reaches cinq: GO is rebound to F5 in the `config.toml` of each
phase.  Escape reaches it only in the two phases of check 2, `pulse/two` and
`darwin/two`, where nothing else plays (checked in every log).  On GNU/Linux
wtype's keymap gives F5 the physical key code that winit calls Escape, which
the `K` lines show (`key: Named(F5)`, `physical_key: Code(Escape)`); the
logical key is F5, and cinq took it as GO.

## Result

Run 43804596396 (commit d470b7a): every job green, and every check below
passed where it ran.

| Check | ubuntu-latest | windows-latest | macos-latest |
| --- | --- | --- | --- |
| 1 JACK ports past nine | passed | skipped, no JACK | skipped, no JACK |
| 2 two devices, Panic stops both | passed | skipped, one VB-CABLE | passed |
| 3 clamp, 16-bit | passed | passed (FlexASIO Int16) | skipped, Core Audio takes float |
| 3 clamp, 24-bit | passed (S24_LE); packed S24_3LE skipped, cinq says it cannot play it | passed (FlexASIO Int24) | skipped |
| 3 clamp, 32-bit and float | passed | passed (FlexASIO Int32) | skipped |
| 4 interface while a device opens | passed | skipped, no slow device | skipped |
| 5 one output on 1-2 of the default device plays | passed (PulseAudio's) | passed (VB-CABLE) | passed (BlackHole 16ch) |
| 5 outputs view and Levels tab with the Vim keys | passed | passed | passed |

### 1 JACK ports past nine (job `jack`)

jackd with its dummy backend offers 16 playback ports.  An output on 10 to 12
is read as two shows, since cinq takes a stereo output on adjacent channels
and a mono one on one: `pair` has Pair on 10-11 and Mono 12 on 12, and
`singles` has M10, M11, and M12 on 10, 11, and 12 at 0, -6, and -12 dB.  The
cue is `stereo.wav`, 1 kHz at 0.3 on the left and 0.15 on the right.

- `jack_lsp -c`: `out_0` to `out_11` of cinq connect to `system:playback_1` to
  `system:playback_12` in order, in both shows.
- `jack_capture` of all 16 ports, in dBFS RMS: `pair` has -13.5, -19.5, and
  -28.0 on ports 10, 11, and 12 (the left, the right, and a mono output at
  -12 dB), and `singles` has -16.0, -22.0, and -28.0; the other 13 ports are
  silent in both.

### 2 two outputs on two devices, and Panic

- GNU/Linux: a private PulseAudio with four null sinks, whose ALSA names are
  `alsa:house` and `alsa:booth`, and one cue that sends to House at 0 dB and
  to Booth at -6 dB.  `parecord` of each monitor: both play, at -13.5 and
  -19.5 dBFS, and both are silent for the last 3.1 and 3.2 seconds of the
  recording after Escape.  The sound lasts 7.3 and 7.2 seconds against 6.4
  seconds between F5 and Escape in cinq's log (recorder lag).
- Darwin: House on BlackHole 16ch and Booth on BlackHole 2ch, each recorded
  with sounddevice: -13.5 and -19.5 dBFS, silent for the last 4.9 seconds
  after Escape.
- Windows: skipped.  LABSN's helper installs one VB-CABLE, and a second WASAPI
  device would need the A+B pack.

### 3 clamping

One output on channel 1, three shows: `single` (a cue of `clip.wav`, 100 Hz at
0.8, peak 0.8), `two` (two cues of it at 0 dB that one GO fires together, sum
1.6), and `boost` (`half.wav`, 100 Hz at 0.5, with a send of +12 dB, 1.99).
`clip.py` reads the recording: a wrap shows as a step past 0.5 between
neighboring samples, where a clean 100 Hz sine steps 0.013 per unit of
amplitude.

- GNU/Linux, in a NixOS virtual machine with the kernel's loopback card (the
  kernels of the runners have no sound), ALSA's hw device held to one format
  under the copy plugin: S16_LE, S24_LE, S32_LE, and FLOAT_LE gave a peak of
  0.8000 for `single` and 1.0000 for `two` and `boost`, with the largest step
  0.0105 to 0.0261 and no wrap.  S24_3LE (packed 24-bit) gave `Unplayable("it
  takes no samples that cinq can play")` at the open, so it is skipped.
- Windows: FlexASIO on VB-CABLE with `sampleType` Int16, Int24, and Int32, the
  same nine takes recorded through WASAPI loopback: the same peaks and steps,
  no wrap.  cinq's log does not print the format that the stream opened in, so
  that FlexASIO took the setting is its configuration's word, not cinq's.
- Darwin and the PulseAudio sink of `s24le` are skipped: Core Audio and ALSA's
  pulse plugin offer float, so cinq never opens an integer stream there.

### 4 opening off the interface thread (job `pulse`)

The real plugin of PulseAudio opens a stream in about 10 ms here and stalls
later, so the script makes each open slow by stopping the server (SIGSTOP) for
two seconds and running it for one, four times, as cinq starts.  The first
open, `alsa:house`, took 1.98 s.  36 keys of `j` and `k`, 250 ms apart, begin
when the window shows: the 7 that fell in the open were each followed by their
update within 0 to 1 ms (the `K` and `U` lines of `pulse/slow.log`).  A show of
four outputs on four devices then opened one after the other without a pause
in the interface.

### 5 Windows and Darwin, and GNU/Linux too

Left on channel 1 and Right on channel 2, on the default device (VB-CABLE
there, BlackHole 16ch after `SwitchAudioSource`) and, as a control, on the
device by its ID.  On Linux it is `alsa:house`.

- Outputs view: `l` shows the outputs, `j` moves, `o` adds one (3 outputs),
  `d d` cuts it (2), `p` pastes it (3), and Control+o goes back to the cue
  list.  The `U` lines read (outputs shown, outputs) as (no, 2), (yes, 2),
  (yes, 3), (yes, 2), (yes, 3), (no, 3), on every runner and phase.
- Sound: both outputs played at -16.0 to -16.6 dBFS (a take that starts a
  little after the sound).
- Levels tab: `g t` twice, Tab five times to the send of Right, select-all
  (the snapshot does not select the text a Tab reaches, so typing `-12`
  appended to `+0.0`), `-12`, and Enter set the send to -12.0 (the `U` line
  reads `Right: -12.0` and `Left: 0.0`), and the second take read -16.0 and
  -28.0, 12 dB apart.

## Runs

| Run | Commit | Result |
| --- | --- | --- |
| 43804596396 | d470b7a | all six jobs green, every check passed where it ran |

## Traps

- Tab does not select the text of a send in this snapshot; use select-all.
- `wtype g g` types a space between its arguments, which would be GO; checks
  that type several keys pass one string.
- A device that takes seconds to open cannot be had from the real PulseAudio
  plugin, whose stall comes after the open; SIGSTOP of the server makes it.
- The logging patch stamps `K` lines with the key as winit read it, so `F5`
  shows a physical key of Escape on GNU/Linux.
