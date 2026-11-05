<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# runner-sandbox: Windows runners

Probed `windows-latest`, `windows-2025`, `windows-2022`, and `windows-11-arm`
for the facts that the Windows section of `SKILL.md` states.

Outcome, in runs 41927402728 and 41943699443: every job passed.

- Every label ran the job as `runneradmin`, elevated, in the active console
  session with a 1024x768 display.  `windows-latest` was Windows Server 2025.
- Git's system configuration set `core.autocrlf true` and
  `core.symlinks true`, and Developer Mode was on, so `actions/checkout` wrote
  `SKILL.md` with CRLF line endings and `AGENTS.md` as a symbolic link.
  `git config --global core.autocrlf false`, or `--system`, before checkout
  kept LF, and so did `* -text` in `lf/.gitattributes`.  `grep -c $'\r$'`
  counted every line in Git Bash whatever the line endings, so the second run
  checked with `git ls-files --eol` instead.
- In Git Bash, `cmd /c echo /usr/bin` started an interactive `cmd`,
  `cmd //c echo /usr/bin` printed `C:/Program Files/Git/usr/bin`, and
  `MSYS_NO_PATHCONV=1` passed `/usr/bin` through.  `ln -s` copied the file,
  and `MSYS=winsymlinks:nativestrict` made a symbolic link.
- Python wrote to a pipe in cp1252 and failed on U+2713 with
  `UnicodeEncodeError`; `PYTHONUTF8=1` fixed it.
- `TEMP` was `C:\Users\RUNNER~1\AppData\Local\Temp`.  The workspace was on
  `D:` on x64, which had 148 GB free against 31 GB on `C:` on `windows-2025`,
  and on `C:` on `windows-11-arm`, which had no `D:`.
- `windows-11-arm` ran Windows 11 Enterprise, with Git Bash as x64 under
  emulation but PowerShell, Python, and Go as arm64, and with Defender
  real-time protection on, which the x64 labels had off.  It had no `docker`
  or `winget`, and `windows-2022` had no `winget`.
