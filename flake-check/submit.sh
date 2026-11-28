#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Writes the commits of a clone of the flake since the base commit to
# flake-check/flake.patch, commits and pushes it on the current branch of
# this sandbox, and prints the ID of the run that the push starts.
#
# Usage: flake-check/submit.sh <flake clone> <message>
set -euo pipefail

clone=$1
msg=$2
repo=${RUNNER_SANDBOX_REPO:-yzhou216/runner-sandbox}
here=$(cd "$(dirname "$0")" && pwd)
base=$(grep -v '^#' "$here/base" | tr -d '[:space:]')
{
  echo "SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>"
  echo
  echo "Changes to github.com/yzhou216/flake under test, as a diff against"
  echo "${base:0:7}, under the terms of that repository:"
  echo
  git -C "$clone" log --reverse --format='  %h %s' "$base..HEAD"
  echo
  git -C "$clone" diff "$base" HEAD
} >"$here/flake.patch"
git -C "$here/.." add flake-check
git -C "$here/.." commit --quiet -m "$msg"
git -C "$here/.." push --quiet -u origin HEAD 2>&1 | grep -v '^remote:' || true
sha=$(git -C "$here/.." rev-parse HEAD)
for _ in $(seq 60); do
  run=$(gh run list -R "$repo" -c "$sha" --json databaseId -q '.[0].databaseId' 2>/dev/null || true)
  [ -n "$run" ] && break
  sleep 5
done
echo "run ${run:-not found} for $sha: https://github.com/$repo/actions/runs/${run:-}"
