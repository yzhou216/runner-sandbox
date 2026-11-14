<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# Copies into an emulated aarch64 FreeBSD machine

cinq's CI copies a closure into an aarch64 FreeBSD 15.1 machine that
`vmactions/freebsd-vm` emulates on `ubuntu-latest`, and that copy failed
about once in 25 jobs, always in rsync ("failed verification", "protocol
incompatibility").  This branch measured how often each way of copying
fails and how.

Run 43632768905: one job built cinq's `aarch64-freebsd` closure at
yzhou216/cinq fe73a0c (103,720,960 bytes), and ten jobs each started the
machine once with `sync: no` and copied the closure 12 times by each method,
480 copies in all, comparing its SHA-256 inside the machine with the
runner's and hashing a mismatch again.

| Method | Intact | Failed, noticed | Corrupt, unnoticed |
|---|---|---|---|
| rsync | 113 | 7 | 0 |
| rsync `--checksum-choice=md5` | 112 | 8 | 0 |
| scp | 105 | 1 | 14 |
| tar over ssh | 89 | 0 | 31 |

Every corrupt copy hashed the same twice, so the damage is on the disk, not
in the hash.  SSH's MAC guards the wire, so the data is damaged inside the
emulated machine, between sshd and the program that writes it, and the
longer the chain of pipes, the more often.  rsync's own checksum notices
every damaged copy, whichever checksum it uses, and no copy that rsync
finished was damaged, so retrying rsync is the fix.  The x86_64 machines,
which run under KVM, never failed in about 290 jobs of cinq's CI.
