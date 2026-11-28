<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq/2026-11-28-r22-fix-nix-drv

drvPaths of every package, check, dev shell, and formatter of cinq's flake on
all 20 systems (175 entries), before (`cinq-before`, f89ccec60), with the
loader fix alone (`cinq-loader`), and with the tidies (`cinq-after`).

- Run 46160241510 (eval, compare): the loader fix changes `deb`, `rpm`, and
  `archlinux` on the seven GNU/Linux systems and nothing else but the `reuse`
  and `treefmt` checks, which read the sources.  The tidies change only those
  two checks.  `cinq`, `doc`, `demo`, `darwin-pkg`, the cross builds, and the
  dev shells keep their drvPaths.
- Run 46136774712 (packages): on x86_64-linux and aarch64-linux, `deb`, `rpm`,
  and `archlinux` of before and after are identical files, and so are the
  files inside.
- Same run (loader): i686-linux, x86_64-linux, and aarch64-linux name the same
  loader before and after.  The other four systems have no cached linker, so
  their new name was not printed; their old name holds the glob.
