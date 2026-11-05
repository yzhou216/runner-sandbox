<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# runner-sandbox: screenshots and videos

Took a screenshot and a five second video of a few windows on GNU/Linux
(`ubuntu-latest`, `ubuntu-24.04`, `ubuntu-26.04-arm`), Darwin
(`macos-latest`, `macos-15`, `macos-26-intel`), and Windows
(`windows-latest`, `windows-2022`, `windows-11-arm`) runners, uploaded them,
and looked at them after `gh run download`.

Outcome, in runs 41927406054, 41943701809, and 41944029867: every job
passed, and every screenshot and video showed what was on the screen.

- GNU/Linux had `Xvfb` and `xvfb-run` but no `ffmpeg` or ImageMagick.  Apt
  installed `ffmpeg` and `x11-apps` in 15 to 43 seconds.  `Xvfb`, started in
  the background in one step, served `xclock` and `xeyes` in the next ones,
  and `ffmpeg -f x11grab` recorded it.
- Darwin recorded the GUI session with `screencapture` and
  `screencapture -V 5` without any grant.  The system TCC database already
  granted Screen Recording to `/bin/bash`, `/opt/hca/hosted-compute-agent`,
  and the runner's provisioner; the job's shell descends from
  `/bin/bash` through `hosted-compute-agent` and the runner.  The displays
  were 1024x768 on arm64 and 1920x1080 on `macos-26-intel`.
- Windows showed a 1024x768 desktop in the job's session.  .NET's
  `CopyFromScreen`, in `screenshot.ps1`, captured it without installing
  anything, and `ffmpeg -f gdigrab` captured it after `choco install ffmpeg`,
  which took 22 to 45 seconds.
- On `windows-11-arm`, a privacy settings prompt covered the desktop.
  Stopping `WWAHost` closed it and opened the Start menu, which Escape
  closed.  In the third run, a `wsl.exe` window asking to update WSL then
  covered Notepad.
