#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Writes the commits of a clone of the flake since the base commit to
# flake-check/series as one patch each, commits and pushes them on the
# current branch, and prints the ID of the series run that the push starts.
#
# Usage: flake-check/submit-series.sh <flake clone> <message>
set -euo pipefail

clone=$1
msg=$2
repo=${RUNNER_SANDBOX_REPO:-yzhou216/runner-sandbox}
here=$(cd "$(dirname "$0")" && pwd)
base=$(grep -v '^#' "$here/base" | tr -d '[:space:]')
rm -rf "$here/series"
git -C "$clone" format-patch --quiet -o "$here/series" "$base..HEAD"
cat >"$here/series/README" <<END
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

Commits to github.com/yzhou216/flake under test, one patch each on top of
${base:0:7}, under the terms of that repository.
END
git -C "$here/.." add -A flake-check/series
git -C "$here/.." commit --quiet -m "$msg"
# GitHub sometimes rejects a push with "(failed)" and accepts it again
for _ in 1 2 3 4 5; do
  git -C "$here/.." push --quiet -u origin HEAD 2>/dev/null && break
  sleep 10
done
sha=$(git -C "$here/.." rev-parse HEAD)
for _ in $(seq 60); do
  run=$(gh run list -R "$repo" -c "$sha" --json databaseId -q '.[0].databaseId' 2>/dev/null || true)
  [ -n "$run" ] && break
  sleep 5
done
echo "run ${run:-not found} for $sha: https://github.com/$repo/actions/runs/${run:-}"
