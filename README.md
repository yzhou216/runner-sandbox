<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq tutorial screenshots

Screenshots for the LaTeX tutorial of
[cinq](https://github.com/yzhou216/cinq), taken from cinq at 6f5bc2e on
`ubuntu-latest`.

## What ran

`shots.yml` builds cinq with `nix build github:yzhou216/cinq/<commit>`, so
the branch holds no copy of its tree.  It checks out cinq's `demo` branch
with Git LFS, the sounds that the tutorial tells its reader to fetch, puts
this branch's `storm.cinq.toml` over the one there, and `media.sh` makes the
title card and the film with ffmpeg's lavfi sources.  `shots.sh` then starts
headless sway with two outputs of 1280x960 at scale 2, so each shot is
2560x1920, 4:3.  It sets `default_border none`, and a `for_window` rule
moves the stage window to the second output.  Each window fills its output,
so each shot holds one whole window and no decorations.  wtype drives cinq
with its vi keys, and grim takes a shot of each output.  Sound goes to a
PulseAudio null sink through ALSA's pulse plugin (`ALSA_PLUGIN_DIR`), which
plays it in real time, so that cues run at their real pace.  The session's
home is `/home/alice`, the path that the help page shows.  oxipng shrinks
the shots losslessly.

## Results

Run 42889099737 took the first set, with sounds that `media.sh` made then,
which the `demo` branch does not hold.  Run 42921403769 took them again for
the tutorial rewritten after QLab's tutorials, with the `demo` branch's
sounds, and passed in about 20 minutes, most of it the Nix build of cinq.
Its artifact `shots` holds `window`, `playing` (after the first GO), `time`,
`levels`, `chain`, `running`, `keys`, `help`, `stage` (the second output),
`showing`, and `added` (a new show, `my-show.cinq.toml`).  Every shot shows
what its step meant to show, with no warnings in the sidebar.  `chain` came
out the same as `playing` but for the progress bar, since the first GO
leaves the fade cue on standby and selected, so the tutorial leaves it out.
The other ten, at 96 to 298 KB each, are those of `doc/tutorial/screenshots`
in cinq.

The film on the `demo` branch is the same film encoded with `-crf 34
-preset slow`, 2.3 MB instead of 15 MB; no shot shows a frame of it.  The
show there also carries `level = 0.0` on the two effects and `advance`
before `level` on the film, as cinq saves it; neither changes what plays.

## Learned

- wtype joins its text arguments with spaces, so `wtype j j` types `j`,
  Space, `j`, and Space is GO in cinq.  Pass a sequence as one argument
  (`wtype jj`), or each key with `-k`.
- Neither `wlrctl pointer click` nor sway's own `seat seat0 cursor press`
  reached cinq's window in a headless sway started with
  `WLR_LIBINPUT_NO_DEVICES=1`, as tried on the local machine before the
  run, so the Show mode check box, which no key toggles, could not be
  clicked.  wtype's keys work since its `-s 300` delay gives cinq time to
  bind the new virtual keyboard, while wlrctl's virtual pointer lives only
  while its command runs.
- At 1024x768 the cue list leaves out its Pre, Post, and Then columns for
  the names of this show; 1280x960 shows them all.
