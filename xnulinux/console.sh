#!/bin/bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Interactive terminal on XNU's /dev/console, like upstream's xnu-console.sh:
# fki.ko logs the console's output, with escape and backspace bytes written
# out as \x1b and \x08, and each line typed here goes to xnu.ko's console_in
# file
#   console.sh [--new]
if [ "${1:-}" = --new ]; then follow=-W; else follow=-w; fi
[ "${XNU_CONSOLE_NOECHO:-0}" = 1 ] && stty -echo
sudo dmesg "$follow" | sed -un '/fki: darwin console: /{s/^.*fki: darwin console: //; s/\\x1b/\x1b/g; s/\\x08/\x08/g; p}' &
trap 'kill $!' EXIT
while IFS= read -r line; do printf '%s\n' "$line" >> /var/tmp/console-in; done
