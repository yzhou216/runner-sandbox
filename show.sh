# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Pieces of a show, written to standard output, which the scripts source.
# output NAME DEVICE CHANNELS [DEVICE_NAME] makes an output, on the default
# device where DEVICE is empty, list opens the one cue list, cue ID NAME
# FILE ADVANCE NAME=LEVEL... makes an audio cue that sends to the outputs
# named at the levels given, and memo ID NAME makes a memo cue.
output() {
  printf '[[output]]\nname = "%s"\n' "$1"
  [ -n "$2" ] && [ "$2" != - ] && printf 'device = "%s"\n' "$2"
  [ -n "${4:-}" ] && [ "$4" != - ] && printf 'device_name = "%s"\n' "$4"
  printf 'channels = "%s"\n\n' "$3"
}
list() { printf '[[list]]\nname = "Main"\n\n'; }
cue() {
  local id=$1 name=$2 file=$3 advance=$4 send
  shift 4
  printf '[[list.cue]]\nid = %s\nnumber = "%s"\nkind = "audio"\nname = "%s"\nfile = "%s"\nadvance = "%s"\n\n[list.cue.sends]\n' \
    "$id" "$id" "$name" "$file" "$advance"
  for send in "$@"; do printf '"%s" = %s\n' "${send%=*}" "${send##*=}"; done
  printf '\n'
}
memo() { printf '[[list.cue]]\nid = %s\nnumber = "%s"\nkind = "memo"\nname = "%s"\n\n' "$1" "$1" "$2"; }
