<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# runner-sandbox

Throwaway repository where agents test on GitHub-hosted runners, one branch
per session, as [SKILL.md](SKILL.md) describes.

To use it, fork it, keep the fork public so that the runners stay free, enable
workflows in the fork's Actions tab, and point agents at the fork.  For
riscv64 jobs, also install the
[RISE RISC-V Runners](https://riscv-runners.riseproject.dev/) GitHub App on
the fork.  Cloning it into an agent's skills directory also installs it as a
skill.

## License

Licensed under `0BSD OR CC0-1.0`.
