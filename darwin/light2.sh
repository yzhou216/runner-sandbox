#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Check 5: light output made to fail by reject routes (a send to a rejected address fails
# at once, where a pf drop leaves the send a success), and NEW fires a light cue by GO and
# then gets no input at all.  A burst of screenshots says when the sidebar shows the
# problem, and when it goes after the routes are removed.  The routes cover the loopback
# address, the broadcast address of each interface, and the sACN groups, which is every
# address that cinq sends to.
#
# Usage: light2.sh APPS OUT
set -uo pipefail
APPS=$1
OUT=$2
# shellcheck source=common2.sh
. darwin/common2.sh

templatelights "$W/tlights"
{ ifconfig -a; netstat -rn -f inet; } > "$OUT/network-before.txt" 2>&1
broadcasts=$(ifconfig -a | awk '/inet .* broadcast / { for (i = 1; i <= NF; i++) if ($i == "broadcast") print $(i + 1) }')
echo "broadcasts: $broadcasts" | tee "$OUT/routes.txt"
# route VERB: adds the reject routes, or deletes them
route_all() {
  flag=""
  [ "$1" = add ] && flag=-reject
  sudo route -n "$1" -host 127.0.0.1 $flag >> "$OUT/routes.txt" 2>&1 || echo "route $1 127.0.0.1 failed" >> "$OUT/routes.txt"
  for b in $broadcasts; do
    sudo route -n "$1" -host "$b" $flag >> "$OUT/routes.txt" 2>&1 || echo "route $1 $b failed" >> "$OUT/routes.txt"
  done
  sudo route -n "$1" -net 239.255.0.0/16 $flag >> "$OUT/routes.txt" 2>&1 || echo "route $1 239.255.0.0/16 failed" >> "$OUT/routes.txt"
}
restore() {
  route_all delete
  sudo ifconfig lo0 down
  sudo ifconfig lo0 up
  ifconfig lo0 >> "$OUT/routes.txt" 2>&1
}
APP=$APPS/cinq-new.app
launch 5-light "$(fresh light tlights)" "$(conf go-new "$GO")"
shot before
focus
burst "$D/frames" &
burster=$!
sleep 3
route_all add
t_rule=$(now)
mark "routes added"
netstat -rn -f inet > "$OUT/network-rules.txt" 2>&1
sleep 3
osascript -e 'tell application "System Events" to key code 101' || true
t_go=$(now)
mark "GO under the routes"
sleep 15
route_all delete
t_lift=$(now)
mark "routes removed"
sleep 15
touch "$D/frames/stop"
wait "$burster"
shot end
# probe: what a UDP send to each address does with the routes back
route_all add
python3 - << 'PY' > "$OUT/probe.txt" 2>&1
import re, socket, subprocess
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
bs = re.findall(r"broadcast (\S+)", subprocess.run(["ifconfig", "-a"], capture_output=True, text=True).stdout)
for to in [("127.0.0.1", 6454), ("127.0.0.1", 5568)] + [(b, 6454) for b in bs] + [("239.255.0.1", 5568)]:
    try:
        print("with the routes", to, "sent", s.sendto(b"x", to))
    except OSError as e:
        print("with the routes", to, "failed:", e)
PY
restore
quit
readframes "$D/frames"
{
  echo "routes added at $t_rule, GO at $t_go"
  python3 darwin/first.py "$D/frames" 'not sent|Warnings \([1-9]' present "$t_rule"
  python3 darwin/first.py "$D/frames" 'not sent|Warnings \([1-9]' present "$t_go"
  echo "routes removed at $t_lift"
  python3 darwin/first.py "$D/frames" 'not sent|Warnings \([1-9]' absent "$t_lift"
} | tee "$OUT/light.txt"
cat "$OUT/routes.txt" "$OUT/probe.txt"
