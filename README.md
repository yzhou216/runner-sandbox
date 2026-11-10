<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq tests on Windows, 2026-11-10

`cinq/` holds a snapshot of cinq's branch `land` (6f5bc2e), the fixes of
the 2026-11-10 review before they went to master, among them a test that
only Windows compiles: shows on two drives or network shares keep their
unsaved changes in two files.  cinq's own CI builds for Windows but runs
no tests there.

The workflow runs `cargo test --locked` natively with the stable toolchain
and `CINQ_SHARE_DIR` empty, so that no scdoc is needed.

| Runner | Run | Result |
| --- | --- | --- |
| windows-latest | 42814050340 | 316 passed |
| windows-11-arm | 42814050340 | 316 passed |

`shows_on_other_drives_keep_their_changes_in_other_files` passed on both.
Run 42812594794 tested an earlier snapshot and was cancelled in favour of
this one.
