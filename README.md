<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# darwin: cinq idle, file watch, light news, exit, and byte order mark on macos-latest

Branch `cinq/2026-11-24-darwin-idle` of yzhou216/runner-sandbox, started from `cinq/2026-11-24-darwin-checks`.  Runner: macos-latest (Darwin 26.7.2, aarch64), cinq built natively with `nix build github:yzhou216/cinq/<SHA>#cinq`, run as cinq.app by `open -n`, keys by System Events, 1024x768, standard error to a file.  NEW `16c365b2c189c8e0b0f21e9d8902383a714bb61c`, OLD `d534417e464a5a95823a183a0483a17a230bf3d3`.  Pictures and logs are in the artifact `darwin-macos-latest` of each run.

Runs (all green): **45462049037** (round one on NEW, idle, file watch, light with pf, exit, byte order mark), **45473153343** (idle with 1 and 2000 files, thread samples, red close button), **45479786428** and **45479585710** (device choice, label found, list not yet opened), **45479879605** (light with reject routes and lo0 down), **45486713563** (device choice with Enter, final).  The runs 45486038455, 45486167722, and 45486111056 were cancelled before they ran, because a continuation session had put `ifconfig en0 down` into `light2.sh`; commit 2e7fc50 removed it.  No run of mine took the network down.

| # | Check | macos-latest |
| - | - | - |
| 1 | Round one again | pass: all eight as in round one (stderr 0 bytes at start; cua message at config.toml:1:10; `sd` added cue 3; OSC `/go`, `no cue is numbered "99"`, port taken; BROKEN and `file not found`; paste `Pasted cue 3`; `Cannot open`/`Cannot save` in words; `SEND FAILED`) |
| 2 | Idle, 500 files | OLD 2.85 and 2.85 %, NEW 1.71 and 1.66 % of one core (two rounds, 60 s each) |
| 3 | Idle, lights demo | OLD 0.78 and 1.21 %, NEW 0.45 and 0.38 % |
| 4 | File watch | pass: BROKEN and `file not found` +1.2 s after the delete, gone +1.1 s after the restore, `0:05.0` for the replaced file +1.1 s (burst of screenshots, frame stamps) |
| 5 | Light news | not possible without cutting the runner's network |
| 6 | Clean exit | pass: red button, process gone within 0.4 s; Command-Q 0.20 to 0.39 s; SIGTERM 0.02 to 0.04 s, for both shows; Command-W does nothing |
| 7 | Byte order mark | pass: silent at both starts, F9 is GO, `audio_device` written after the mark |

## Details

- **Idle control** (45473153343): with 1 file OLD 2.09 %, NEW 1.38 %; with 2000 files OLD 4.86 %, NEW 2.01 %.  A 10 s `sample` of the 500-file show: NEW's main thread was in `mach_msg2_trap` in all 6939 samples (never woke); OLD's main thread ran event handlers in 30 to 39 samples.  What NEW still uses is not the window: the CoreAudio IO thread runs cinq's mixer render callback.  The lights demo, which also holds an audio cue, uses 0.4 %, and I did not find why the audio shows use 1.4 %.
- **Check 5**: a pf rule dropping UDP to ports 5568 and 6454 left `sendto` successful (45462049037, `probe.txt`), so cinq saw nothing fail.  Reject routes to the broadcast address and the sACN group plus `lo0 down` (45479879605) made the loopback and broadcast sends fail (`Can't assign requested address`, `No route to host`), but the sACN multicast send went out through the interface address and still succeeded, so a round always had one packet out, and cinq (correctly, by its rule that one packet out is out) showed no warning in 294 frames.  Failing the multicast needs `en0` down, which cuts the runner.
- **Local Network prompt**: `ui click Allow` after GO in the lights demo found nothing for OLD or NEW (0 presses in 25 s, `prompts.txt`); `after-go.png` shows no prompt.  It did press Allow once in some launches (3-prompt-old, 3-lights-old-1, 6-500-w, 6-lights-w, 5-light), but I took no picture of the prompt, so I cannot say whose it was.  Since 6-500-w sends nothing to the LAN, I doubt all of them were cinq's.
- **Check 7**: hint `f` labels the Audio device list `sk` only after scrolling the sidebar (`ui scroll 890 380 -3`), since it sits below the window's bottom edge; `sk` focuses the list, Enter opens it.  File after the choice: `audio_device = "coreaudio:NullAudioDevice_UID"` first, then `[keys]`, with the bytes `ef bb bf` ahead of it.
- Traps: `ps -o nlwp` does not exist on Darwin; Command-W is not bound in cinq.
