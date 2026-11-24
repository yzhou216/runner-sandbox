<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq 16c365b vs d534417 (v0.1.7), idle fixes, on GNU/Linux

Branch `cinq/2026-11-24-linux-idle`, workflow `.github/workflows/idle.yml`, scripts `checks/check.sh` (round one,
unchanged but for the SHA) and `checks/idle.sh`.  Both builds come from `nix build github:yzhou216/cinq/<SHA>#cinq`
(and `#demo`) on `ubuntu-latest` (x86_64) and `ubuntu-24.04-arm` (aarch64), and run in headless sway (pixman,
tiny-skia) with wtype and grim.  GO is also bound to F9.

Runs: 45461625671 (round one on NEW, both arches green; its idle numbers are void, see below), 45479703430,
45485536986, **45492686801** (final, both arches green; checks 2 to 7).  45485392105 was cancelled for a script change.
Outputs of the final run are in `results/<arch>/`.

## Sound device

The null ALSA device of round one never blocks, so cinq's `cpal_alsa_out` thread spun at 100 percent of a core in
every build, which hid the idle numbers (run 45461625671: 100.1 to 101.5 percent for OLD and NEW alike).  The idle
checks use an ALSA `file` device on a named pipe that `pv -L 192000` drains, so it is paced like hardware; the audio
threads then idle.  `snd-dummy` is not in the runners' kernels.

## Verdicts (final run, same in both arches unless noted)

| # | Check | x86_64 | aarch64 | Evidence |
| --- | --- | --- | --- | --- |
| 1 | Round one, eight checks, NEW | pass | pass | run 45461625671, `round-one-steps.txt`; no verdict differs from round one |
| 2 | Idle, 500 cues | pass, less than OLD, not near zero | pass, less than OLD, not near zero | table below |
| 3 | Idle, lights demo | pass | pass | table below |
| 4 | File watch | pass | pass | delete: screen changed 1.0 and 0.9 s after the move (State `BROKEN`, sidebar `1 Cue 1: file not found`); restore: 0.5 and 0.4 s (warning gone); replace by `mv`: 1.6 and 1.4 s (Action `0:00.7` to `0:05.0`) |
| 5 | Light news | pass | pass | see below |
| 6 | Clean exit | pass | pass | window closed with `swaymsg kill`; process gone after 0.03 to 0.04 s, 500-cue show and lights demo, OLD and NEW |
| 7 | Byte order mark | pass | pass | stderr 0 bytes at both starts; F9 started the cue; `audio_device = "alsa:paced"` chosen in the list (hint `sc`, Return, Down, Control+y); file ends `efbbbf`, then `audio_device`, `[keys]`; second start silent |

## Idle processor time over 60 s, percent of one core (three runs each, then mean)

| Show | x86_64 OLD | x86_64 NEW | aarch64 OLD | aarch64 NEW |
| --- | --- | --- | --- | --- |
| 500 audio cues, nothing playing | 1.18, 1.08, 1.13 (1.13) | 0.68, 0.73, 0.73 (0.71) | 0.82, 0.85, 0.87 (0.85) | 0.33, 0.35, 0.35 (0.34) |
| lights demo, nothing playing | 0.23, 0.27, 0.25 (0.25) | 0.15, 0.10, 0.13 (0.13) | 0.27, 0.27, 0.25 (0.26) | 0.15, 0.12, 0.13 (0.13) |
| lights demo, cue 1 held | 0.27 | 0.15 | 0.33 | 0.15 |

NEW is lower everywhere, but the 500-cue show is not near zero.  The main (UI) thread of OLD used 24 to 32 ticks in
60 s and NEW's main thread shows none among its busy threads, which is the fix.  What remains in NEW, and is the same in
OLD, is one thread that sleeps 1 ms in a loop: `strace` shows 2733 `clock_nanosleep` calls of 1 ms in 3 s
(`results/*/new-500-strace.txt`), about 950 wakes per second, 34 ticks per minute on x86_64, 12 on aarch64.  It only
appears with the 500-cue show (the lights demo has one wake per second less than OLD and no such thread); it is probably
the streaming decoder of the sound made ready for the standby cue, but I did not trace it to its source.

## Light news (NEW, lights demo, network namespace with `lo` down)

`ip netns` with `sudo` (the namespace is not `unshare -rn`, which Ubuntu's AppArmor limits; the Wayland socket is
reachable across namespaces).  The runner's own interfaces are untouched.

- Failure: cinq launched in the namespace, no input at all: the sidebar showed `Light output: no network interface`
  in the first frames, 0.05 to 0.11 s after launch (stderr line at +0.05 s), before any GO; GO later changed the
  screen at once (cue 1 fired, warning stays).  Screenshots `5-lo-boot-*.png`, `5-dummy-boot-*.png`.
- End by bringing `lo` up (127.0.0.1): warning gone 0.83 to 0.87 s after, with no input.
- End by bringing up a dummy interface `d0` (10.77.0.1/24, broadcast 10.77.0.255) with `lo` still down: warning
  gone 0.79 to 0.86 s after, with no input.

## Not the same as the brief

- Check 5 failed from the start of the show, not only after GO: the patch's zero levels count as due.
- No `snd-dummy`; the paced pipe device stands in (see above).
