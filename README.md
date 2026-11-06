<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: the problems of config.toml, drawn by miette as it is edited

Recorded cinq (`cinq/`, a snapshot of uncommitted work on fc1bdce) reporting
the problems of its `config.toml` as vim edits the file while cinq runs, on
GNU/Linux (headless sway on `ubuntu-latest`), with `demo.sh`.  vim edits the
file in the terminal at the top left, cinq runs from the terminal under it,
which shows what cinq writes to standard error, and cinq's window is on the
right, with the warnings in its sidebar and the keys in effect in its status
bar.  A band under each video captions each step.

- `1-editing.mp4`, 52 seconds: cinq starts with two mistakes in the file and
  draws both with miette, each with the lines around it and the part that it
  lies in marked, while its sidebar lists them by line.  Fixing one and
  saving reports the other, and fixing that reports nothing.  Deleting a
  closing bracket reports that the file is not TOML, with a mark past the end
  of the line, and cinq falls back to the vi keys; undoing brings back the
  keys of the file.
- `2-paused.mp4`, 31 seconds: Control+s pauses the output of the terminal.  A
  save with a mistake, and then its fix, both reach the window at once while
  the terminal stays paused, since a thread of the watch's own writes to
  standard error.  Control+q resumes the terminal, and the report that waited
  appears.

`demo.sh` starts the sway session itself, so it ran the same on a GNU/Linux
machine with the tools of `nix build --inputs-from ./cinq` before it ran
here.

## Outcome

Run 42262739356 (0f288fa) passed, and both recordings show every step as the
captions tell it, with the file at `/home/runner/.config/cinq/config.toml`.
The `videos` artifact of the run holds them, along with the times of the
captions and the logs of sway, foot, and wf-recorder.
