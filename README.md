<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: picture cues that go up fullscreen

Recorded cinq with picture cues and a stage window that each GO of a
picture or video cue makes fullscreen, from a snapshot of cinq in
`cinq/`, running `pictures.cinq.toml` by the steps of `steps.txt`:
GO puts a preshow picture up fullscreen, GO starts a film that covers it
while a fade cue fades the picture out under it, the fullscreen key brings
the stage back to a window, the film ends on black, GO puts the
intermission picture up fullscreen again, and a panic fades it out.
`media.sh` makes the PNG, the JPEG, and the film with ffmpeg.

- GNU/Linux (`ubuntu-latest`): headless sway with two outputs, a control
  display and a projector that a `for_window` rule moves the stage to, each
  recorded with `wf-recorder -o` and stacked side by side.
- Darwin (`macos-latest`): one display; the fullscreen key is F11, which
  Darwin keeps for Show Desktop, so the script sets `AXFullScreen` of the
  stage window to false, as the green button does.
- Windows (`windows-latest` and `windows-11-arm`): one display, with
  `pictures.ps1`, which presses each key in whichever window of cinq is in
  front, so that it never raises the main window over the fullscreen stage.

## Outcome

Run 42299944366 (fba1e48) records the commit rebased onto cinq's config.toml
problems with miette (3b4c026): every job passed, Windows arm64 on its
second attempt, after its first could not start ffmpeg, which Chocolatey
had installed, before the walkthrough began.  The run before it,
42294757320 (2cb5124), passed on GNU/Linux and Windows, and failed on
Darwin when System Events could not find the stage for a moment right
after it left fullscreen (-1728); the script now logs such a failure
rather than ending the step, and the next step found the stage again.

Run 42190035036 (54d22a2), before the rebase: every job passed, and every recording shows the
whole walkthrough as intended.  The stage opens as a window, goes
fullscreen with the first GO, shows the film over the preshow, comes back
to a window with the fullscreen key while the film plays, stays black once
the film ends, since the fade stopped the preshow, goes fullscreen again
with the intermission, and fades to black with the panic.  The state of the
stage that each step logs follows it: on GNU/Linux a step after the one
that changes it, since sway applies fullscreen a moment after the key.

Notes for later sessions:

- `screencapture -V` on Darwin gave the last frame of the movie a duration
  of about 15 minutes, which ffmpeg's `fps` filter repeats to the end; cut
  the encoding with `-t` at the length recorded.
- On `windows-11-arm` the prompt to update WSL opened again a second after
  the panic, over everything, so the recording that went to the user was
  cut at 27.3 seconds.
- Leaving fullscreen on Darwin restores the stage at the size it opened at,
  not the size that the script gave it through System Events.
