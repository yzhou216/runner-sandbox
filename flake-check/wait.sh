#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Waits up to eight minutes for a run, then prints the results of its
# jobs, or that it is still running; call it again in that case.
#
# Usage: flake-check/wait.sh <run id>
set -uo pipefail

run=$1
repo=${RUNNER_SANDBOX_REPO:-yzhou216/runner-sandbox}
timeout 480 gh run watch "$run" -R "$repo" --interval 30 >/dev/null 2>&1
state=$(gh run view "$run" -R "$repo" --json status,conclusion -q '.status + " " + .conclusion')
echo "run $run: $state"
case $state in
  completed*) ;;
  *) echo "still running; call wait.sh again"; exit 0 ;;
esac
dir=$(mktemp -d)
gh run download "$run" -R "$repo" -D "$dir" >/dev/null 2>&1 || true
# A series run has one artifact per commit; its summary is enough here
if [ -f "$dir/results-summary/results.tar" ]; then
  tar -xf "$dir/results-summary/results.tar" -C "$dir/results-summary"
  cat "$dir/results-summary/results/results.md"
  echo "artifacts: $dir"
  exit 0
fi
for t in "$dir"/*/results.tar; do
  [ -f "$t" ] || continue
  d=$(dirname "$t")
  tar -xf "$t" -C "$d"
  cat "$d/results/results.md"
  echo
done
echo "artifacts: $dir"
gh run view "$run" -R "$repo" --json jobs -q '.jobs[] | "job \(.name): \(.conclusion)"'
