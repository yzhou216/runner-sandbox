#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Checks 2 and 3: processor time of cinq with nothing playing, for OLD and NEW, with
# the 500-cue show and with the lights demo.  Two rounds, OLD then NEW in each, in one job.
#
# Usage: idle.sh APPS OUT
set -uo pipefail
APPS=$1
OUT=$2
# shellcheck source=common2.sh
. darwin/common2.sh

template500 "$W/t500"
templatelights "$W/tlights"

# Local Network prompt of each build: a GO sends Art-Net to the LAN
for v in old new; do
  APP=$APPS/cinq-$v.app
  launch "3-prompt-$v" "$(fresh "prompt-$v" tlights)" "$(conf "go-$v" "$GO")"
  code 101
  answer "$v"
  shot after-go
  quit
done

for round in 1 2; do
  for v in old new; do
    APP=$APPS/cinq-$v.app
    launch "2-files-$v-$round" "$(fresh "files-$v-$round" t500)" "$W/empty"
    shot loaded
    idle "check 2, 500 files, $v, round $round"
    quit
  done
  for v in old new; do
    APP=$APPS/cinq-$v.app
    launch "3-lights-$v-$round" "$(fresh "lights-$v-$round" tlights)" "$W/empty"
    idle "check 3, lights demo, $v, round $round"
    shot idle
    quit
  done
done
cat "$OUT/idle.txt" "$OUT/prompts.txt"
