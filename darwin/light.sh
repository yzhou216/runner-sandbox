#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Check 5: light output made to fail by pf, with a rule that drops only UDP to the
# Art-Net and sACN ports (the runner's network stays up), and NEW fires a light cue
# by GO and then gets no input at all.  A burst of screenshots says when the sidebar
# shows the problem, and when it goes after the rule is lifted.
#
# Usage: light.sh APPS OUT
set -uo pipefail
APPS=$1
OUT=$2
# shellcheck source=common2.sh
. darwin/common2.sh

templatelights "$W/tlights"
printf 'block drop quick proto udp from any to any port { 5568, 6454 }\n' > "$OUT/pf-block.conf"
printf '# no rules\n' > "$OUT/pf-none.conf"
# probe: what a UDP send to each address and port does
probe() {
  python3 - << 'PY'
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
for to in [("127.0.0.1", 6454), ("127.0.0.1", 5568), ("255.255.255.255", 6454), ("239.255.0.1", 5568), ("127.0.0.1", 9)]:
    try:
        print(to, "sent", s.sendto(b"x", to))
    except OSError as e:
        print(to, "failed:", e)
PY
}
APP=$APPS/cinq-new.app
launch 5-light "$(fresh light tlights)" "$(conf go-new "$GO")"
sudo pfctl -E > "$OUT/pfctl.txt" 2>&1
token=$(sed -n 's/^Token : //p' "$OUT/pfctl.txt")
{ echo "before the rule"; probe; } > "$OUT/probe.txt" 2>&1
# shellcheck disable=SC2024 # The file is the runner's own
sudo pfctl -f "$OUT/pf-block.conf" >> "$OUT/pfctl.txt" 2>&1
sudo pfctl -s rules >> "$OUT/pfctl.txt" 2>&1
{ echo "with the rule"; probe; } >> "$OUT/probe.txt" 2>&1
cat "$OUT/probe.txt"
shot before
focus
burst "$D/frames" &
burster=$!
sleep 4
osascript -e 'tell application "System Events" to key code 101' || true
t_go=$(now)
mark "GO under the rule"
sleep 15
# shellcheck disable=SC2024 # The file is the runner's own
sudo pfctl -f "$OUT/pf-none.conf" >> "$OUT/pfctl.txt" 2>&1
t_lift=$(now)
mark "rule lifted"
sleep 15
touch "$D/frames/stop"
wait "$burster"
shot end
sudo pfctl -f /etc/pf.conf >> "$OUT/pfctl.txt" 2>&1 || true
[ -n "$token" ] && sudo pfctl -X "$token" >> "$OUT/pfctl.txt" 2>&1
quit
readframes "$D/frames"
{
  echo "GO at $t_go"
  python3 darwin/first.py "$D/frames" 'not sent|Warnings \([1-9]' present "$t_go"
  echo "rule lifted at $t_lift"
  python3 darwin/first.py "$D/frames" 'not sent|Warnings \([1-9]' absent "$t_lift"
} | tee "$OUT/light.txt"
