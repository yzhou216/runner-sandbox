---
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0
name: runner-sandbox
description: Run tests on GitHub-hosted runners (GNU/Linux, Windows, and Darwin, on x64 and arm64) and RISE RISC-V runners through a public sandbox repository, one branch per session.  Use when work needs an operating system or architecture the local machine lacks, such as testing a Darwin program from GNU/Linux.
---

# runner-sandbox

This repository is a throwaway sandbox for testing on GitHub-hosted runners.
Its default branch holds only these instructions and a workflow that reviews
pull requests.  Every session works on a branch of its own, which stays behind
as the record of what was tested and as a worked example for later sessions.

## Rules

- This repository must stay public, which keeps the standard runners free.
  Everything pushed, logged, or uploaded is public too, so it must contain no
  secrets.
- Use standard runners, or the RISE runners under Runners.  Larger runners
  are billed even for public repositories.
- Never commit to the default branch, merge into it, or touch another
  session's branch.
- License files you write under `0BSD OR CC0-1.0`, with SPDX headers like the
  ones in this repository.  Files taken from a project keep its license.

## Session

Run every `gh` command with `-R "$repo"`, where `repo` is the sandbox you were
pointed to, as `owner/name`.  In a clone of a fork, `gh` otherwise targets the
parent repository.

1. Clone the default branch into a fresh directory and create a branch named
   `<project>/<YYYY-MM-DD>-<topic>`, always with the date in UTC:

   ```sh
   repo=owner/runner-sandbox
   dir=$(mktemp -d)/runner-sandbox
   gh repo clone "$repo" "$dir" -- --depth 1
   cd "$dir" && git switch -c "myproject/$(date -u +%F)-darwin-build"
   ```

2. Add what to test, such as prebuilt binaries, a source snapshot, or
   scripts, and one workflow in `.github/workflows/` that runs `on: push`;
   `workflow_dispatch` and `schedule` only trigger workflows on the default
   branch.  Check the workflow with `actionlint` (`nix run nixpkgs#actionlint`
   on Nix) before pushing.  A run that fails at once with a workflow file
   issue has a syntax error, such as an unquoted colon and space in a one-line
   `run`.

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

5. Finish with a `README.md` that records what was tested, on which runners,
   and the outcome, with run IDs.  Commit it with `[skip ci]` in the message,
   so that pushing starts no runs, then push and leave the branch.

If commits land on the default branch by mistake, push them to the session
branch and restore the default branch to its last commit before them:

```sh
git push origin HEAD:refs/heads/<session-branch>
git push --force-with-lease origin <last-clean-commit>:<default-branch>
```

## Improving this skill

If a session finds a trick that later sessions would need, such as a runner
quirk and its workaround, propose it for this skill.  With your user's
approval, open a pull request against
[yzhou216/runner-sandbox](https://github.com/yzhou216/runner-sandbox) that
changes only `SKILL.md` and cites the session branch and run IDs that show the
trick.  After finishing the session, branch from that repository's default
branch:

```sh
upstream=yzhou216/runner-sandbox
git fetch --depth 1 "https://github.com/$upstream" master
git switch -c "runner-sandbox/$(date -u +%F)-<topic>" FETCH_HEAD
```

Edit and commit `SKILL.md`, then push the branch to the sandbox and open the
pull request:

```sh
git push -u origin HEAD
gh pr create -R "$upstream" --head "${repo%/*}:$(git branch --show-current)" \
  --title "runner-sandbox: <summary>" --body "<trick, session branch, run IDs>"
```

## Runners

Choose labels from GitHub's current list of
[standard runners for public repositories](https://docs.github.com/en/actions/reference/runners/github-hosted-runners#standard-github-hosted-runners-for-public-repositories),
which changes as images come and go; `actionlint` reports labels newer than
its copy of the list as unknown.  What a label runs, such as the release
behind a `-latest` label, is in the
[runner-images](https://github.com/actions/runner-images) repository.

For riscv64, the [RISE RISC-V runners](https://riscv-runners.riseproject.dev/)
are free on a sandbox with the RISE GitHub App installed.  Choose a label
marked generally available in RISE's
[list of labels](https://riscv-runners.riseproject.dev/docs/getting-started/labels#labels).
A runner has taken about two minutes to pick up a job; a job still queued
after several minutes likely means the app is missing or the label is wrong,
so cancel it with `gh run cancel` and tell your user.  List the label in
`.github/actionlint.yaml` for `actionlint` to accept it:

```yaml
self-hosted-runner:
  labels: [<label>]
```

## Notes

- Set `defaults.run.shell: bash` so every step runs Bash with `pipefail`, Git
  Bash on Windows included.  Otherwise Windows steps run PowerShell, and other
  steps run `bash -e` without `pipefail`, so `cmd | tail` reports the status
  of `tail`.
- Use `strategy.fail-fast: false` in matrices, so that a failing job does not
  cancel the rest of the matrix.
- Tar directories before uploading them with `actions/upload-artifact`, which
  drops file modes.  Artifacts uploaded with `archive: false` cannot be
  fetched with `gh run download`; use
  `gh api "repos/$repo/actions/artifacts/<id>/zip" > <file>`.
- Nix installs with `cachix/install-nix-action` on GNU/Linux and Darwin
  runners.  If a build needs more disk space than a GitHub-hosted GNU/Linux
  runner has free, run `wimpysworld/nothing-but-nix` before it; otherwise skip
  it to save time.

### Screenshots and videos

Upload captures with `actions/upload-artifact` and view them after
`gh run download`; `ffmpeg -i video.mp4 -vf fps=1 frame-%02d.png` splits a
video into one image per second.  The last two commands of each block below
take a screenshot and five seconds of video.

- GNU/Linux runners have no display, but have `Xvfb`.  Start it in the
  background, where it outlives the step, and capture it with `ffmpeg`, which
  apt installs in under a minute:

  ```sh
  Xvfb :99 -screen 0 1280x800x24 &
  echo DISPLAY=:99 >> "$GITHUB_ENV"
  sudo apt-get update && sudo apt-get install -y ffmpeg
  ffmpeg -f x11grab -video_size 1280x800 -i :99 -frames:v 1 screenshot.png
  ffmpeg -f x11grab -video_size 1280x800 -framerate 15 -t 5 -i :99 \
    -pix_fmt yuv420p video.mp4
  ```

- On Darwin, `screencapture` records the GUI session without a grant of its
  own, since the system TCC database grants Screen Recording to the runner:

  ```sh
  screencapture -x screenshot.png
  screencapture -x -V 5 video.mov
  ```

- Windows runners show a 1024x768 desktop, which `ffmpeg` captures after
  `choco install ffmpeg -y`:

  ```sh
  ffmpeg -f gdigrab -i desktop -frames:v 1 screenshot.png
  ffmpeg -f gdigrab -framerate 15 -t 5 -i desktop -pix_fmt yuv420p video.mp4
  ```

### Darwin

- System Integrity Protection is off, and the `runner` user has a logged-in
  GUI session, so launchd agents and event taps work.  Check SIP with
  `csrutil status`, and load an agent with
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

  TCC checks the responsible process, which for anything started from the
  job's shell is the runner, so start the program as a launchd agent for TCC
  to check the program itself.  Restart it after the grant: a running process
  keeps its earlier answer.
- `swiftc` is available for small helpers, such as posting or watching input
  events with CoreGraphics.
- Linkers sign arm64 binaries ad hoc, but not x64 ones, for which
  `codesign -dv` fails.

### Windows

- Jobs run as `runneradmin`, elevated, in the console session, so windows
  that a step opens show on the desktop.
- Git's system configuration sets `core.autocrlf true` and
  `core.symlinks true`, and Developer Mode is on, so `actions/checkout` writes
  text files with CRLF line endings and symbolic links as links.  To keep the
  committed line endings, run `git config --global core.autocrlf false` before
  checkout, or commit a `.gitattributes` with `* -text`.  Check with
  `git ls-files --eol`, since `grep $'\r$'` in Git Bash matches every line.
- Git Bash rewrites arguments that look like POSIX paths for native
  programs: `cmd /c echo /usr/bin` starts an interactive `cmd`, and
  `cmd //c echo /usr/bin` prints `C:/Program Files/Git/usr/bin`.  Set
  `MSYS_NO_PATHCONV=1` to pass arguments unchanged.  Git Bash's `ln -s`
  copies the file unless `MSYS=winsymlinks:nativestrict` is set.
- Python writes to a pipe in cp1252 and raises `UnicodeEncodeError` on
  characters outside it; set `PYTHONUTF8=1`.
- `TEMP` is the 8.3 short path `C:\Users\RUNNER~1\AppData\Local\Temp`, which
  Git Bash shows as `/tmp`, so resolve paths before comparing them.
- On x64, the workspace is on `D:`, which has more free space than `C:`.
- On arm64, there is only `C:`, Git Bash runs as x64 under emulation while
  PowerShell, Python, and Go run natively, and Defender real-time protection
  is on.  A privacy settings prompt has also covered the desktop;
  `Stop-Process -Name WWAHost` closes it but opens the Start menu, which
  Escape closes, and a prompt to update WSL has appeared over other windows.

### RISC-V

- RISE runners are Docker containers in which jobs run as `runner` with
  `sudo`, and Docker works.  On the `scaleway-em-rv1` board, they have 4 CPUs
  of `rv64imafdcsu`, which lacks the vector extension, 15 GiB of memory, and
  76 GB of free disk.
- `clang` and `node` are not on `PATH`, though JavaScript actions such as
  `actions/checkout` run.
- `cachix/install-nix-action` installs Nix, but cache.nixos.org has few
  riscv64-linux builds: `nix build nixpkgs#hello --dry-run` lists 133
  derivations to build.
