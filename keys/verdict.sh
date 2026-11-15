#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Merges the marks of each step with the lines that cinq logged, by time, into
# vi.timeline and cua.timeline, and judges each check from them.  A check
# that finds no lines to judge from fails.
#
# Usage: verdict.sh OUT, with vi.err, vi.marks, cua.err, and cua.marks in OUT
set -uo pipefail

out=$1

for phase in vi cua; do
  [ -f "$out/$phase.err" ] || continue
  {
    awk '/^SANDBOX / { t = $2; $1 = ""; $2 = ""; sub(/^ +/, ""); print t, "L", $0 }' "$out/$phase.err" | tr -d '\r'
    awk '$2 == "MARK" { t = $1; $1 = ""; $2 = ""; sub(/^ +/, ""); print t, "M", $0 }' "$out/$phase.marks" | tr -d '\r'
  } | sort -s -n -k1,1 | awk '$2 == "M" { sub(/^[^ ]+ M /, ""); print "=== " $0; next } { sub(/^[^ ]+ L /, ""); print }' > "$out/$phase.timeline"
done

# segment PHASE LABEL: lines after every step with that label, up to the next
segment() {
  awk -v label="=== $2" '
    /^=== / { keep = ($0 == label); next }
    keep { print }' "$out/$1.timeline"
}
# state PHASE LABEL: list, rows, selected row, and selected cue after the
# last step with that label
state() { segment "$@" | grep -o 'list=[0-9]* rows=[0-9]* selected_row=[^ ]* selected_id=[^ ]*' | tail -n 1; }
# number NAME STATE: value of the field NAME in a state, without Some()
number() { printf '%s\n' "$2" | sed -n "s/.*$1=\(Some(\)\{0,1\}\([0-9]*\).*/\2/p"; }
has_step() { grep -q "^=== $2\$" "$out/$1.timeline"; }

pass() { printf 'PASS %s\n' "$1"; }
fail() { printf 'FAIL %s: %s\n' "$1" "$2"; }
# check NAME STATUS DETAIL: passes when STATUS is 0
check() {
  if [ "$2" = 0 ]; then pass "$1"; else fail "$1" "$3"; fi
}

if [ -f "$out/vi.timeline" ]; then
  v() { segment vi "$@"; }
  s() { state vi "$@"; }

  # 1. Control+i is Control+i, not Tab, and goes forward after Control+o went back
  raw=$(v 1-ctrl-i | grep '^raw key')
  printf '%s\n' "$raw" | grep -q 'raw key=Character("i").*CTRL' && ! printf '%s\n' "$raw" | grep -q 'Named(Tab)'
  check "1 Control+i arrives as the character i with Control, not as Tab" $? "$raw"
  v 1-ctrl-o | grep -q 'action=Some((Back' && v 1-ctrl-i | grep -q 'action=Some((Forward'
  check "1 the keymap reads Control+o as Back and Control+i as Forward" $? "$(v 1-ctrl-o | grep '^press'; v 1-ctrl-i | grep '^press')"
  g=$(s 1-G); o=$(s 1-ctrl-o); i=$(s 1-ctrl-i)
  [ -n "$g" ] && [ "$g" = "$i" ] && [ "$g" != "$o" ]
  check "1 selection after l, G, Control+o, Control+i: [$(number selected_row "$g")] [$(number selected_row "$o")] [$(number selected_row "$i")]" $? "G [$g], back [$o], forward [$i]"

  # 3. Control+d and Control+u move the selection by half the visible rows
  before=$(number selected_row "$(s 3-gg)")
  after_d=$(number selected_row "$(s 3-ctrl-d)")
  after_u=$(number selected_row "$(s 3-ctrl-u)")
  rows=$(number rows "$(s 3-ctrl-d)")
  d=$(v 3-ctrl-d | grep '^page HalfDown')
  u=$(v 3-ctrl-u | grep '^page HalfUp')
  fit=$(printf '%s\n' "$d" | sed -n 's/.* fit=\([0-9]*\).*/\1/p')
  moves=$(printf '%s\n' "$d" | sed -n 's/.* moves=\([0-9]*\).*/\1/p')
  up_moves=$(printf '%s\n' "$u" | sed -n 's/.* moves=\([0-9]*\).*/\1/p')
  half=$(( ${fit:-0} / 2 ))
  want_d=$(( ${before:-0} + half ))
  [ "$want_d" -gt "${rows:-0}" ] && want_d=${rows:-0}
  want_u=$(( ${after_d:-0} - half ))
  [ "$want_u" -lt 1 ] && want_u=1
  [ "${fit:-0}" -ge 2 ] && [ "$moves" = "$half" ] && [ "$after_d" = "$want_d" ]
  check "3 Control+d: $fit rows visible, moves $moves (half is $half), row $before to $after_d" $? "wanted row $want_d; $d"
  [ "${fit:-0}" -ge 2 ] && [ "$up_moves" = "$half" ] && [ "$after_u" = "$want_u" ]
  check "3 Control+u: moves $up_moves (half is $half), row $after_d to $after_u" $? "wanted row $want_u; $u"

  # 4. y y, p, d d, u, and 3 u act on the cue list
  r() { number rows "$(s "$1")"; }
  yy=$(r 4-yy); ppp=$(r 4-p); dd=$(r 4-dd); u1=$(r 4-u); u3=$(r 4-3u)
  [ "$yy" = 5 ] && [ "$ppp" = 8 ] && [ "$dd" = 7 ] && [ "$u1" = 8 ] && [ "$u3" = 5 ]
  check "4 rows: after y y $yy (5), p p p $ppp (8), d d $dd (7), u $u1 (8), 3 u $u3 (5)" $? "rows after each step"
  v 4-yy | grep -q 'action=Some((Copy' && v 4-p | grep -q 'action=Some((PasteBelow' && v 4-dd | grep -q 'action=Some((Cut' && v 4-u | grep -q 'action=Some((Undo' && v 4-3u | grep -q 'message Undo(3)'
  check "4 the keymap reads y y as Copy, p as PasteBelow, d d as Cut, u as Undo, and 3 u as Undo(3)" $? "$(for l in 4-yy 4-p 4-dd 4-u 4-3u; do v "$l" | grep '^press.*action=Some' | tail -n 1; done)"

  # 5. Control+o opens no file dialog, Control+l does, and Cancel closes it
  ! v 5-ctrl-o | grep -q 'message Open'
  check "5 Control+o sends no Open" $? "$(v 5-ctrl-o | grep -i open)"
  v 5-ctrl-l | grep -q 'message Open'
  check "5 Control+l sends Open" $? "$(v 5-ctrl-l)"
  if [ -f "$out/vi-5-windows.txt" ]; then
    grep -q 'Open' "$out/vi-5-windows.txt" && ! grep -q 'Open' "$out/vi-5-windows-after.txt"
    check "5 the file dialog showed after Control+l and is gone after the cancel" $? "$(cat "$out/vi-5-windows.txt" "$out/vi-5-windows-after.txt" 2>&1)"
  fi
  if grep -q 'Escape' "$out/vi.timeline" "$out/cua.timeline" 2> /dev/null; then
    fail "5 no Escape reached cinq" "$(grep -n Escape "$out/vi.timeline" "$out/cua.timeline" | head -n 3)"
  else
    pass "5 no Escape reached cinq"
  fi
  if grep -q 'action=Some((Go' "$out/vi.timeline" "$out/cua.timeline" 2> /dev/null; then
    fail "no Space reached cinq" "$(grep -n 'action=Some((Go' "$out/vi.timeline" "$out/cua.timeline" | head -n 3)"
  else
    pass "no Space reached cinq"
  fi

  # 6. The back and forward buttons of the mouse
  last_g=$(s 6-G)
  previous=$last_g
  for pair in "6-back:Back" "6-forward:Forward" "6-side:Back" "6-extra:Forward" \
    "6-xbutton1-back:Back" "6-xbutton2-forward:Forward" "6-button3-back:Back" "6-button4-forward:Forward"; do
    label=${pair%%:*}
    direction=${pair##*:}
    has_step vi "$label" || continue
    now=$(s "$label")
    v "$label" | grep -q "mouse button pressed $direction" && v "$label" | grep -q "message SidePressed(.*, $direction)" && [ -n "$now" ] && [ "$now" != "$previous" ]
    check "6 $label: reaches cinq as $direction, row $(number selected_row "$previous") to $(number selected_row "$now")" $? "$(v "$label" | head -n 6)"
    previous=$now
  done
fi

if [ -f "$out/cua.timeline" ]; then
  c() { segment cua "$@"; }
  # 2. Alt+ArrowLeft and Alt+ArrowRight go back and forward in the cua keymap
  end=$(state cua 2-end)
  for pair in "2-alt-left:Back" "2-alt-right:Forward" "2-system-events-alt-left:Back" "2-system-events-alt-right:Forward"; do
    label=${pair%%:*}
    direction=${pair##*:}
    has_step cua "$label" || continue
    now=$(state cua "$label")
    c "$label" | grep -q "action=Some(($direction" && [ -n "$now" ]
    check "2 $label: reads as $direction, row $(number selected_row "$end") to $(number selected_row "$now")" $? "$(c "$label" | grep '^raw\|^press' | head -n 4)"
    end=$now
  done
fi
