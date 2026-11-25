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

## Final rerun with cinq ab9fada0, run 45520340417

Branch `cinq/2026-11-24-audio-final` of yzhou216/runner-sandbox; run 45520340417 (ubuntu-latest, success, first attempt).  Build ab9fada06db75110f94d58cd471a965a55448af9; only the SHA in audio.yml changed.  Earlier numbers: README of `cinq/2026-11-24-audio-rerun` (16c365b2, run 45461092221).

| # | Check | Earlier | FINAL |
|---|-------|---------|-------|
| 1 | -6 dB cue, default device | -12.0, booth silent | -12.0, booth silent |
| 2 | sends House 0, Booth -10 | -12.0 / -22.0 | -12.0 / -22.0 |
| 3 | fade to -20 over 3 s | -18 to -26 (booth -28 to -36), flat at +3.0 s | same |
| 4 | loop, devamp | loop -9.0, left at +8.0 s, ending -17.0, silent +10.0 s | same (devamp mark +7.3 s) |
| 5 | Stop / Panic | 0.1 s; 1.0 s | Stop silent by 0.2 s (house) / 0.1 s (booth); Panic silent at 1.0 s |
| 6 | -12 typed while playing | -12 to -18, -22 to -28 | same |
| 7 | booth device gone | house -6.0; cue 9 OUTPUT FAILED; "alsa:booth: cannot open"; "Booth (cannot open)" | same, seen in screenshots |
| 8 | last send cleared | status "Changed cue 8 to play nowhere", silent 0.1 s | same |

Differences: none.  Every number and message matches.  Only wobble: Stop on the house sink reads last sound 0.1 s, silent from 0.2 s, against 0.1 s before; the same 0.2 s measuring accuracy covers it.
