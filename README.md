<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: audio device list

This branch records cinq choosing its audio device from the Audio device
list in its sidebar.  `cinq/` holds the tree of cinq that was tested, which
was uncommitted work on top of yzhou216/cinq commit
3de2258bb8fd33a3a4cefaff22cade29d3a9e450.

## What runs

`.github/workflows/picker.yml` builds cinq with `nix build ./cinq`, makes
the media of `devices.cinq.toml` with `media.sh`, and runs `demo.sh`.

- `demo.sh` starts a PulseAudio server with two null sinks, `house` and
  `booth`, which `~/.asoundrc` names as the ALSA devices `alsa:house`
  ("House speakers") and `alsa:booth` ("Booth monitors").  The default sink
  is `house`.
- A headless sway session holds the main window on HEADLESS-1 and the stage
  window on HEADLESS-2, each 1280x960.  `wf-recorder` records each output,
  and `parecord` records the monitor of each sink.
- The script plays the audio cue, moves to the list with Tab, opens it
  with Enter, moves to Booth monitors with j, and chooses it with Enter.  It
  then plays the video cue, chooses House speakers again, and presses
  Escape for Panic.  It finds the row of each device from
  `cinq --audio-devices`.
- `marks.tsv` holds the time of each step and the captions.
  `config-*.toml` and `inputs-*.txt` hold `config.toml` and
  `pactl list sink-inputs` after each step.

`compose.sh` was run locally on the downloaded artifact.  It puts the two
outputs side by side, the waveform of each sink under them, and the
captions below.  The house sink plays in the left channel and the booth
sink in the right.  It lines up the sound with the picture by the onset of
the music after GO, since `parecord` started recording about 1.5 seconds
after it was started.

## Results

Run 42941260844 on `ubuntu-latest` passed in about 25 minutes, most of it
spent building cinq.

- `cinq --audio-devices` listed `alsa:null`, `alsa:house`, and
  `alsa:booth`.
- After the first choice, `config.toml` held
  `audio_device = "alsa:booth"`, and the stream of cinq played on sink 1
  (`booth`).  After the second choice, it held
  `audio_device = "alsa:house"`, and the stream played on sink 0 (`house`).
- The music moved to the booth with no silence between the two sinks.  It
  moved back to the house after about 1.6 seconds of silence.
- On the local machine, the same script gave about 1.2 seconds of silence
  on the first move.  A new cpal stream on PulseAudio's ALSA plugin took
  three callbacks at once and then none for about 1.9 seconds, whatever
  its buffer size.  The sounds that move start there from where they were
  as the stream opened, so they play late by that silence.
