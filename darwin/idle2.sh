#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Where the idle time of cinq goes: the idle measure with 1 and 2000 files, and a
# sample of the threads of the 500-file show, OLD and NEW.
#
# Usage: idle2.sh APPS OUT
set -uo pipefail
APPS=$1
OUT=$2
# shellcheck source=common2.sh
. darwin/common2.sh

templaten "$W/t1" 1
templaten "$W/t2000" 2000
template500 "$W/t500"
for n in 1 2000; do
  for v in old new; do
    APP=$APPS/cinq-$v.app
    launch "2-files$n-$v" "$(fresh "files$n-$v" "t$n")" "$W/empty"
    idle "$n file(s), $v"
    quit
  done
done
for v in old new; do
  APP=$APPS/cinq-$v.app
  launch "2-sample-$v" "$(fresh "sample-$v" t500)" "$W/empty"
  sleep 25
  sample "$PID" 10 -file "$D/sample.txt" > "$D/sample-out.txt" 2>&1
  quit
done
cat "$OUT/idle.txt"
