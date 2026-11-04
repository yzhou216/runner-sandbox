---
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0
name: runner-sandbox
description: Run tests on GitHub-hosted runners (GNU/Linux, Windows, and Darwin, on x64 and arm64) through a public sandbox repository, one branch per session.  Use when work needs an operating system or architecture the local machine lacks, such as testing a Darwin program from GNU/Linux.
---

# runner-sandbox

This repository is a public, throwaway sandbox for testing on GitHub-hosted
runners.  Its default branch holds only these instructions.  Every session
works on a branch of its own, which stays behind as the record of what was
tested.

## Rules

- This repository must stay public, which keeps the standard runners free, so
  everything pushed, logged, or uploaded is public too: never push secrets.
- Use standard runners only.  Larger runners are billed even for public
  repositories.
- Never commit to the default branch, merge into it, or touch another
  session's branch.
- License files you write under `0BSD OR CC0-1.0`, with SPDX headers like the
  ones in this repository.  Files taken from a project keep its license.

## Session

Run every `gh` command with `-R "$repo"`, where `repo` is the sandbox you were
pointed to, as `owner/name`.  In a clone of a fork, `gh` otherwise targets the
parent repository.

1. Clone the default branch into a fresh directory and create a branch named
   `<project>/<YYYY-MM-DD>-<topic>`:

   ```sh
   repo=owner/runner-sandbox
   dir=$(mktemp -d)/runner-sandbox
   gh repo clone "$repo" "$dir" -- --depth 1
   cd "$dir" && git switch -c "myproject/$(date +%F)-darwin-build"
   ```

2. Add what to test, such as prebuilt binaries, a source snapshot, or
   scripts, and one workflow in `.github/workflows/` that runs `on: push`.
   `workflow_dispatch` and `schedule` only work for workflows on the default
   branch.  Check the workflow with `actionlint` (`nix run nixpkgs#actionlint`
   on Nix) before pushing; a run that fails at once with a workflow file issue
   has invalid YAML, such as an unquoted colon and space in a one-line `run`.

3. Commit, push, and follow the run of that commit:

   ```sh
   git push -u origin HEAD
   sha=$(git rev-parse HEAD)
   until run=$(gh run list -R "$repo" -c "$sha" \
     --json databaseId -q '.[0].databaseId') && [ -n "$run" ]; do
     sleep 5
   done
   gh run watch "$run" -R "$repo" --exit-status
   gh run view "$run" -R "$repo" --log-failed
   ```

4. Fix and push again on the same branch until the results are clear.

5. Finish by committing a `README.md` that records what was tested, on which
   runners, and the outcome, with run IDs.  Add `[skip ci]` to that commit
   message so that pushing it starts no runs, push, and leave the branch.

If commits land on the default branch by mistake, push them to the session
branch and restore the default branch to its last commit before them:

```sh
git push origin HEAD:refs/heads/<session-branch>
git push --force-with-lease origin <last-clean-commit>:<default-branch>
```

## Runners

Choose labels from GitHub's current list of
[standard runners for public repositories](https://docs.github.com/en/actions/reference/runners/github-hosted-runners#standard-github-hosted-runners-for-public-repositories),
which changes as images come and go; `actionlint` lags behind it.  What a
label runs, such as the release behind a `-latest` label, is in the
[runner-images](https://github.com/actions/runner-images) repository.

## Notes

- Set `defaults.run.shell: bash` so every step runs Bash with `pipefail`, Git
  Bash on Windows included.  Otherwise steps run `bash -e` without `pipefail`,
  where `cmd | tail` reports the status of `tail`, and PowerShell on Windows.
- Use `strategy.fail-fast: false` in matrices, so that one failing runner
  leaves the others running.
- Tar directories before uploading them with `actions/upload-artifact`, which
  drops file modes.  Artifacts uploaded with `archive: false` cannot be
  fetched with `gh run download`; use
  `gh api "repos/$repo/actions/artifacts/<id>/zip" > <file>`.
- Nix installs with `cachix/install-nix-action` on GNU/Linux and Darwin
  runners, after `wimpysworld/nothing-but-nix` frees disk space.
- Branches other than the default branch are earlier sessions, whose
  `README.md` and workflow make worked examples.

### Screenshots and videos

Capture the screen to see what a graphical program does, upload the files
with `actions/upload-artifact`, and look at them after `gh run download`.
`ffmpeg -i video.mp4 -vf fps=1 frame-%02d.png` turns a video into one image
per second.  The last two commands of each block below take a screenshot and
five seconds of video.

- GNU/Linux runners have no display, but have `Xvfb`.  Start it in the
  background, where it outlives the step, and record it with `ffmpeg`, which
  apt installs in under a minute:

  ```sh
  Xvfb :99 -screen 0 1280x800x24 &
  echo DISPLAY=:99 >> "$GITHUB_ENV"
  sudo apt-get update && sudo apt-get install -y ffmpeg
  ffmpeg -f x11grab -video_size 1280x800 -i :99 -frames:v 1 screenshot.png
  ffmpeg -f x11grab -video_size 1280x800 -framerate 15 -t 5 -i :99 \
    -pix_fmt yuv420p video.mp4
  ```

- Darwin runners record their GUI session without a TCC grant, since the
  system database already grants Screen Recording to the runner's processes:

  ```sh
  screencapture -x screenshot.png
  screencapture -x -V 5 video.mov
  ```

- Windows runners show a 1024x768 desktop, which `ffmpeg` records after
  `choco install ffmpeg -y`:

  ```sh
  ffmpeg -f gdigrab -i desktop -frames:v 1 screenshot.png
  ffmpeg -f gdigrab -framerate 15 -t 5 -i desktop -pix_fmt yuv420p video.mp4
  ```

### Darwin

- Darwin runners have had System Integrity Protection off and a logged-in GUI
  session for the `runner` user, so launchd agents and event taps work.
  Confirm with `csrutil status`, and load an agent with
  `launchctl bootstrap gui/$(id -u) <plist>`.
- Grant a privacy permission, here Accessibility, by writing the system TCC
  database:

  ```sh
  sudo sqlite3 "/Library/Application Support/com.apple.TCC/TCC.db" "
    INSERT OR REPLACE INTO access (service, client, client_type,
      auth_value, auth_reason, auth_version, flags)
    VALUES ('kTCCServiceAccessibility', '/usr/local/bin/program', 1,
      2, 4, 1, 0);"
  ```

  TCC checks the responsible process, so start the program as a launchd
  agent rather than from the job's shell, and restart it after the grant:
  a running process keeps its earlier answer.
- `swiftc` is available for small helpers, such as posting or watching input
  events with CoreGraphics.
- Linkers sign arm64 binaries ad hoc, but not x64 ones, for which
  `codesign -dv` fails.

### Windows

- Jobs run as `runneradmin`, elevated, in the console session, so windows
  that a step opens show on the desktop.
- Git's system configuration sets `core.autocrlf true`, so `actions/checkout`
  writes text files with CRLF line endings.  Keep them as committed with
  `git config --global core.autocrlf false` before checkout, or with
  `* -text` in a committed `.gitattributes`.  Check with
  `git ls-files --eol`, since `grep` in Git Bash matches `\r$` on every line.
- Symbolic links check out as links, since Git's system configuration also
  sets `core.symlinks true` and Developer Mode is on.  `ln -s` in Git Bash
  copies the file unless `MSYS=winsymlinks:nativestrict` is set.
- Git Bash rewrites arguments that look like POSIX paths for native
  programs: `cmd /c echo /usr/bin` starts an interactive `cmd`, and
  `cmd //c echo /usr/bin` prints `C:/Program Files/Git/usr/bin`.  Set
  `MSYS_NO_PATHCONV=1` to pass arguments unchanged.
- Python encodes output to a pipe as cp1252, so printing other characters
  fails with `UnicodeEncodeError`; set `PYTHONUTF8=1`.
- `TEMP` is the 8.3 short path `C:\Users\RUNNER~1\AppData\Local\Temp`, so
  resolve paths before comparing them.
- The workspace is on `D:`, which has more free space than `C:`, except on
  `windows-11-arm`, which has only `C:`.
- `windows-11-arm` runs Git Bash as x64 under emulation, but PowerShell,
  Python, and Go natively, and has Defender real-time protection on, unlike
  the x64 labels.  Windows from the first logon have covered its desktop: a
  privacy settings prompt, which `Stop-Process -Name WWAHost` closes, then
  the Start menu, and a prompt to update WSL.
