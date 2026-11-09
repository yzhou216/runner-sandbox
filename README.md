<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: light cues driving lamps drawn from Art-Net

Recorded cinq's light cues (1798b71 on `wip/light-cues`, built straight from
GitHub) driving three lamps on GNU/Linux (headless sway on `ubuntu-latest`),
with `demo.sh`.  No lighting hardware takes part: `view.py`, in the terminal
on the left, receives the Art-Net that cinq sends to 127.0.0.1 on UDP port
6454 and draws the first channels of each universe as lamps, each as bright
as its channel.  cinq's window on the right runs `show.toml` with the Light
Patch tab open, whose Now column shows the level of each instrument.  A band
under the video captions each step.

| Step | Lamps (Front, Back, Cyc) |
| --- | --- |
| GO on cue 1, which fades Front up | 70%, 0%, 0% |
| Cue 2, which names Back and Cyc only | 70%, 80%, 100% |
| Cue 3 fading Front and Back out, stopped halfway by panic | 36%, 41%, 100%, held |
| Cue 4, a blackout | 0%, 0%, 0% |

The lamps and the Now column agreed throughout, both on the runner and on
the machine where `demo.sh` ran first.  To watch cinq's output elsewhere,
run `python3 view.py 3` there and press GO in cinq.

Run 42553804423 (13 minutes), artifact `videos` with `light.mp4`.
