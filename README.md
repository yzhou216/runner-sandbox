<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# runner-sandbox

Skill that lets LLM agents build and test on GNU/Linux, Darwin, and Windows,
on amd64 and aarch64, and on riscv64, from any machine, through free hosted
runners

Each session pushes a branch to a public sandbox repository, follows its
workflow runs, and leaves a `README.md` that records the outcome.
[SKILL.md](SKILL.md) holds the instructions, and branches other than `master`
record earlier sessions.

## Setup

Fork this repository, then clone it into the agent's skills directory:

```sh
gh repo fork yzhou216/runner-sandbox --clone=false --default-branch-only
gh repo clone yzhou216/runner-sandbox <skills-dir>/runner-sandbox -- --depth 1
```

- Keep the fork public, which keeps the standard runners free.  Everything
  that sessions push, log, or upload is public.
- The skill uses the repository named `runner-sandbox` in the user's account.
  To use another, set `RUNNER_SANDBOX_REPO` to its `owner/name`.
- For riscv64 jobs, install the
  [RISE RISC-V Runners](https://riscv-runners.riseproject.dev/) GitHub App on
  the fork.
- If the fork's Actions tab reports that workflows are disabled, enable them
  there.
- Agents without skill support can read `AGENTS.md` instead.  Run `git pull`
  in the clone to update the skill.

## Contributing

With the user's approval, sessions propose new findings as pull requests that
change `SKILL.md` and cite the runs that support them.

## License

Licensed under `0BSD OR CC0-1.0`.
