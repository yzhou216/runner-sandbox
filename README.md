<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# runner-sandbox: standard runner labels

Checked that every standard runner label listed on 2026-11-04 gets a runner,
and the facts that `SKILL.md` states about shells and Darwin runners.

Outcome, in run 41910548046: all 18 labels ran.

- `ubuntu-latest` was Ubuntu 26.04, and `ubuntu-slim` had one CPU; the others
  had 3 (Darwin arm64) or 4.
- The default shell was `bash -e {0}` without `pipefail` on GNU/Linux and
  Darwin, and PowerShell 7.6 on Windows.  `shell: bash` ran
  `bash --noprofile --norc -e -o pipefail {0}` everywhere, Git Bash on
  Windows.
- Every Darwin label, `xcode-27` and the Intel ones included, had System
  Integrity Protection disabled, ran as uid 501, and had `swiftc`.
- The first push failed at once with a workflow file issue: an unquoted `: `
  in a one-line `run:` is invalid YAML.  `actionlint` finds this, though its
  label list lags new images such as `ubuntu-26.04` and `xcode-27`.
