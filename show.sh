#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Writes the show of the run to standard output: House L and House R on
# channels 1 and 2 of the device of ID and NAME, which play, Over on
# channels OVER of it, past those it has, which must change, and Stage L
# and Stage R on a Dante Virtual Soundcard that is not there, which cinq
# tries again, and a cue of a tone of 1 kHz that sends to all five.
#
# Usage: show.sh ID NAME OVER MISSING, with MISSING the ID of the missing
# soundcard on the platform
set -euo pipefail
id=$1 name=$2 over=$3 missing=$4
output() {
  printf '[[output]]\nname = "%s"\ndevice = "%s"\ndevice_name = "%s"\nchannels = "%s"\n\n' "$@"
}
output "House L" "$id" "$name" 1
output "House R" "$id" "$name" 2
output Over "$id" "$name" "$over"
output "Stage L" "$missing" "Dante Virtual Soundcard" 1
output "Stage R" "$missing" "Dante Virtual Soundcard" 2
printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\nkind = "audio"\nname = "Tone"\nfile = "tone.wav"\n\n'
printf '[list.cue.sends]\n"House L" = 0.0\n"House R" = -6.0\nOver = -6.0\n"Stage L" = 0.0\n"Stage R" = 0.0\n'
