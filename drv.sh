#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints "kind system output drvPath" for every package, check, and dev shell
# of every system of the flake under $1, and the formatter's, sorted, to $2.
# An output that does not evaluate prints the last line of its error.
set -uo pipefail
dir=$1 out=$2
flake=./$dir
names() { nix eval --json "$flake#$1" --apply builtins.attrNames | jq -r '.[]'; }
scrub() { sed -E 's#/nix/store/[a-z0-9]{32}-#/nix/store/HASH-#g; s#cinq-(before|loader|after)#cinq-VARIANT#g'; }
export flake
one() {
  kind=$1 system=$2 output=$3
  value=$(nix eval --raw "$flake#$kind.$system${output:+.$output}.drvPath" 2>&1 | tail -n 1 | scrub)
  echo "$kind $system ${output:--} $value"
}
export -f one scrub
{
  for kind in packages checks devShells; do
    for system in $(names "$kind"); do
      nix eval --json "$flake#$kind.$system" --apply builtins.attrNames |
        jq -r --arg kind "$kind" --arg system "$system" '.[] | "\($kind) \($system) \(.)"'
    done
  done
  for system in $(names formatter); do echo "formatter $system"; done
} | xargs -P 4 -L 1 bash -c 'one "$@"' _ | LC_ALL=C sort > "$out"
wc -l "$out"
