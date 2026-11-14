#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# trial.sh ROUNDS: copies closure.tar into the FreeBSD virtual machine
# ROUNDS times by each method and compares its SHA-256 there with the
# runner's.  A mismatch is hashed again, to tell a corrupt copy on disk
# from a hash that the emulated processor got wrong.
set -u
sum=$(sha256sum closure.tar | cut -d' ' -f1)
for round in $(seq "$1"); do
  for method in rsync rsync-md5 scp tar; do
    ssh freebsd 'rm -rf /tmp/t && mkdir /tmp/t'
    start=$(date +%s)
    case $method in
      rsync) rsync -a -e ssh closure.tar freebsd:/tmp/t/ ;;
      rsync-md5) rsync -a --checksum-choice=md5 -e ssh closure.tar freebsd:/tmp/t/ ;;
      scp) scp -q closure.tar freebsd:/tmp/t/ ;;
      tar) tar -cf - closure.tar | ssh freebsd 'tar -xf - -C /tmp/t' ;;
    esac > log 2>&1
    status=$?
    first=$(ssh freebsd 'sha256 -q /tmp/t/closure.tar' 2>/dev/null)
    if [ "$status" -ne 0 ]; then
      result="error($status) $(tail -n 1 log | tr ' ' _)"
    elif [ "$first" = "$sum" ]; then
      result=ok
    else
      second=$(ssh freebsd 'sha256 -q /tmp/t/closure.tar' 2>/dev/null)
      if [ "$second" = "$sum" ]; then result=hash-wrong; else result=corrupt; fi
    fi
    echo "$method $round $result $(($(date +%s) - start))s" | tee -a results.txt
  done
done
