<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# Can the GHC bootstrap chain skip a rung?

[andreabedini/ghc-bootstrap](https://github.com/andreabedini/ghc-bootstrap)
climbs from GHC 6.2.2 through 6.6, 6.10.4 and 6.12.3, and says each rung is
the newest release that accepts the previous one as its boot compiler.  The
minimum boot versions in each release's `configure.ac` allow two shortcuts:
6.4 to 6.8.3 check no minimum, and 6.12.3 needs only 6.8, so
`6.2.2 -> 6.8.3 -> 6.12.3` would replace `6.2.2 -> 6.6 -> 6.10.4 -> 6.12.3`.
The other floor that differs, 9.14.1's 9.6, the repository already reports as
stale.

`default.nix` builds `haskell.compiler.ghc622Bootstrap` of
[nixpkgs#591060](https://github.com/NixOS/nixpkgs/pull/591060), GHC 6.8.3
with it, and GHC 6.12.3 with that, using the chain's settings for 6.6 and
6.12.3 (C flags for gcc 16, unregisterised, no GHCi) and no source changes.
The workflow runs one `ubuntu-latest` job per compiler and passes each
compiler's closure to the next as an artifact.

## Outcome

Run 44850634014: `ghc622` succeeded (3 h 50 min, artifact `ghc622-closure`
kept 30 days), `ghc683` failed, `ghc6123` was skipped.

6.2.2 compiles 6.8.3's stage 1 until `typecheck/TcBinds.lhs`:

```
Couldn't match `BoxyType -> TcM a' against `TcM (HsWrapper, MatchGroup TcId)'
Probable cause: `tcMatchesFun' is applied to too many arguments in the call
```

A boot compiler older than 6.4 reads the hand-written `.hi-boot-6` interfaces
instead of the `.lhs-boot` files, and 6.8's were no longer kept up to date:
`TcMatches.hi-boot-6` lacks the `Bool` argument that `tcMatchesFun` gained,
and 27 `.hi-boot-6` files stand for 30 `.lhs-boot` files.  6.8.1 and 6.8.2
ship the same files.  So no 6.8 release builds with 6.2.2 without editing its
boot interfaces, and the repository's ladder is the shortest without source
changes.  9.14.1's `ghci.cabal.in` requires `ghc-internal >= 9.1001.0`,
which first ships with 9.10, as the repository says.
