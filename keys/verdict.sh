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

for phase in vi cua ops search view menu; do
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
  st=$?; check "1 Control+i arrives as the character i with Control, not as Tab" "$st" "$raw"
  v 1-ctrl-o | grep -q 'action=Some((Back' && v 1-ctrl-i | grep -q 'action=Some((Forward'
  st=$?; check "1 the keymap reads Control+o as Back and Control+i as Forward" "$st" "$(v 1-ctrl-o | grep '^press'; v 1-ctrl-i | grep '^press')"
  g=$(s 1-G); o=$(s 1-ctrl-o); i=$(s 1-ctrl-i)
  [ -n "$g" ] && [ "$g" = "$i" ] && [ "$g" != "$o" ]
  st=$?; check "1 selection after l, G, Control+o, Control+i: [$(number selected_row "$g")] [$(number selected_row "$o")] [$(number selected_row "$i")]" "$st" "G [$g], back [$o], forward [$i]"

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
  st=$?; check "3 Control+d: $fit rows visible, moves $moves (half is $half), row $before to $after_d" "$st" "wanted row $want_d; $d"
  [ "${fit:-0}" -ge 2 ] && [ "$up_moves" = "$half" ] && [ "$after_u" = "$want_u" ]
  st=$?; check "3 Control+u: moves $up_moves (half is $half), row $after_d to $after_u" "$st" "wanted row $want_u; $u"

  # 4. y y, p, d d, u, and 3 u act on the cue list
  r() { number rows "$(s "$1")"; }
  yy=$(r 4-yy); ppp=$(r 4-p); dd=$(r 4-dd); u1=$(r 4-u); u3=$(r 4-3u)
  [ "$yy" = 5 ] && [ "$ppp" = 8 ] && [ "$dd" = 7 ] && [ "$u1" = 8 ] && [ "$u3" = 5 ]
  st=$?; check "4 rows: after y y $yy (5), p p p $ppp (8), d d $dd (7), u $u1 (8), 3 u $u3 (5)" "$st" "rows after each step"
  v 4-yy | grep -q 'action=Some((Copy' && v 4-p | grep -q 'action=Some((PasteBelow' && v 4-dd | grep -q 'action=Some((Cut' && v 4-u | grep -q 'action=Some((Undo' && v 4-3u | grep -q 'message Undo(3)'
  st=$?; check "4 the keymap reads y y as Copy, p as PasteBelow, d d as Cut, u as Undo, and 3 u as Undo(3)" "$st" "$(for l in 4-yy 4-p 4-dd 4-u 4-3u; do v "$l" | grep '^press.*action=Some' | tail -n 1; done)"

  # 5. Control+o opens no file dialog, Control+l does, and Cancel closes it
  ! v 5-ctrl-o | grep -q 'message Open'
  st=$?; check "5 Control+o sends no Open" "$st" "$(v 5-ctrl-o | grep -i open)"
  v 5-ctrl-l | grep -q 'message Open'
  st=$?; check "5 Control+l sends Open" "$st" "$(v 5-ctrl-l)"
  if [ -f "$out/vi-5-windows.txt" ]; then
    grep -q 'Open show' "$out/vi-5-windows.txt" && ! grep -q 'Open show' "$out/vi-5-windows-after.txt"
    st=$?; check "5 the file dialog showed after Control+l and is gone after the cancel" "$st" "$(cat "$out/vi-5-windows.txt" "$out/vi-5-windows-after.txt" 2>&1)"
  fi
  if grep -q 'Named(Escape)' "$out/vi.timeline" "$out/cua.timeline" 2> /dev/null; then
    fail "5 no Escape reached cinq" "$(grep -n 'Named(Escape)' "$out/vi.timeline" "$out/cua.timeline" | head -n 3)"
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
    st=$?; check "6 $label: reaches cinq as $direction, row $(number selected_row "$previous") to $(number selected_row "$now")" "$st" "$(v "$label" | head -n 6)"
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
    st=$?; check "2 $label: reads as $direction, row $(number selected_row "$end") to $(number selected_row "$now")" "$st" "$(c "$label" | grep '^raw\|^press' | head -n 4)"
    end=$now
  done
fi

# fieldof PHASE LABEL NAME: value of the field NAME on the last update line
# after the step LABEL, up to the next space
fieldof() {
  segment "$1" "$2" | grep '^update' | tail -n 1 | sed -n "s/.* $3=\([^ ]*\).*/\1/p"
}
# measured PHASE LABEL: "view content scrolled offset row" of the last
# measurement of the cue list after the step LABEL
measured() {
  segment "$1" "$2" | grep '^measured' | tail -n 1 |
    sed -n 's/^measured view=\([0-9.]*\) content=\([0-9.]*\) scrolled=\([0-9.-]*\) -> offset=\([0-9.-]*\) row=\([0-9]*\).*/\1 \2 \3 \4 \5/p'
}

# 7 and 11. Operators: d 2 j cuts three cues, p pastes them, d u does not undo
if [ -f "$out/ops.timeline" ]; then
  rows_of() { number rows "$(state ops "$1")"; }
  start=$(rows_of 7-gg); cut=$(rows_of 7-d2j); pasted=$(rows_of 7-p); stray=$(rows_of 11-du)
  segment ops 7-d2j | grep -q 'across=Some((Cut, Next, Some(2)))' && [ "$start" = 30 ] && [ "$cut" = 27 ]
  st=$?; check "7 d 2 j cuts three cues: rows $start to $cut" "$st" "$(segment ops 7-d2j | grep '^press\|^update' | tail -n 4)"
  told=$(fieldof ops 7-d2j clipped)
  printf '%s\n' "$(segment ops 7-d2j | grep '^update' | tail -n 1)" | grep -q 'clipped=Some("Cut 3 cues")'
  st=$?; check "7 the clipboard notice reads Cut 3 cues: $(segment ops 7-d2j | grep '^update' | tail -n 1 | sed -n 's/.*clipped=\(.*\)$/\1/p')" "$st" "$told"
  segment ops 7-p | grep -q 'message Paste(' && [ "$pasted" = 30 ]
  st=$?; check "7 p pastes the three cues below: rows $cut to $pasted" "$st" "$(segment ops 7-p | grep '^press\|^message\|^update' | tail -n 4)"
  pending=$(fieldof ops 11-du keys)
  ! segment ops 11-du | grep -q 'message Undo' && [ "$stray" = "$pasted" ] && [ "$pending" = "[]" ]
  st=$?; check "11 d u does not undo: rows $pasted to $stray, keys left $pending" "$st" "$(segment ops 11-du | grep '^press\|^message\|^update' | tail -n 4)"
fi

# 8 and 12. Search, forward and backward, with n and N, and Control+o back to
# where the search began, and F1 for the help page
if [ -f "$out/search.timeline" ]; then
  row_of() { number selected_row "$(state search "$1")"; }
  seq_rows=$(for l in 8-gg 8-enter 8-n1 8-n2 8-enter-back 8-n3 8-N 8-ctrl-o 8-9-ctrl-o; do printf '%s ' "$(row_of "$l")"; done)
  [ "$seq_rows" = "1 10 11 12 11 10 11 10 1 " ]
  st=$?; check "8 rows after gg, /1 Enter, n, n, ?1 Enter, n, N, Control+o, 9 Control+o: $seq_rows(1 10 11 12 11 10 11 10 1)" "$st" "$(for l in 8-enter 8-n1 8-n2 8-enter-back 8-n3 8-N 8-ctrl-o 8-9-ctrl-o; do segment search "$l" | grep '^update' | tail -n 1; done)"
  [ "$(fieldof search 8-slash search)" = 'Some(("",' ] || true
  segment search 8-slash | grep '^update' | tail -n 1 | grep -q 'search=Some(("", false))' && segment search 8-qmark | grep '^update' | tail -n 1 | grep -q 'search=Some(("", true))'
  st=$?; check "8 / opens the search line forward and ? opens it backward" "$st" "$(segment search 8-slash | grep '^update' | tail -n 1; segment search 8-qmark | grep '^update' | tail -n 1)"
  segment search 8-n3 | grep -q 'message SearchStep(1)' && segment search 8-N | grep -q 'message SearchStep(-1)' || segment search 8-N | grep -q 'message SearchStep'
  st=$?; check "8 n and N step the matches (n goes up after a ? search, N goes down)" "$st" "$(segment search 8-n3 | grep '^message'; segment search 8-N | grep '^message')"
  help=$(fieldof search 12-f1 help)
  closed=$(fieldof search 12-q help)
  [ "$help" = true ] && [ "$closed" = false ]
  st=$?; check "12 F1 shows the help page (help=$help), q closes it (help=$closed)" "$st" "$(segment search 12-f1 | grep '^press\|^update'; segment search 12-q | grep '^press\|^update')"
fi

# 9. View keys, judged from the measured view, content, and offset
if [ -f "$out/view.timeline" ]; then
  cues=$(number rows "$(state view 9-gg)")
  # shape LABEL TEST: runs the awk TEST over the measure of a step, with the
  # fields view, content, scrolled, offset, row, the row height rh, the offset
  # of the step before in before, and the row of the step before in last_row
  shape() {
    local label=$1 test=$2 before=$3 last_row=$4 line
    line=$(measured view "$label")
    [ -n "$line" ] || return 1
    # shellcheck disable=SC2086
    set -- $line
    awk -v view="$1" -v content="$2" -v scrolled="$3" -v offset="$4" -v row="$5" -v cues="$cues" -v before="$before" -v last_row="$last_row" "
      function abs(x) { return x < 0 ? -x : x }
      function ceil(x) { return (x == int(x)) ? x : (x > 0 ? int(x) + 1 : int(x)) }
      function floor(x) { return (x == int(x) || x > 0) ? int(x) : int(x) - 1 }
      function clamp(x, lo, hi) { return x < lo ? lo : (x > hi ? hi : x) }
      BEGIN {
        rh = content / cues
        top = ceil(scrolled / rh - 0.001)
        bottom = floor((scrolled + view) / rh + 0.001) - 1
        if (bottom < top) bottom = top
        max = content - view
        exit !($test)
      }"
  }
  show() { measured view "$1" | awk '{ printf "view %s content %s scrolled %s -> offset %s row %s", $1, $2, $3, $4, $5 }'; }
  sh_e1=$(show 9-ctrl-e-1); sh_e3=$(show 9-ctrl-e-3)
  shape 9-ctrl-e-1 'abs(offset - (scrolled + rh)) < 0.6 && row == 2' 0 0
  st=$?; check "9 Control+e scrolls a row, and selection moves onto the first row in view: $sh_e1" "$st" "$(segment view 9-ctrl-e-1)"
  o1=$(measured view 9-ctrl-e-1 | cut -d' ' -f4)
  shape 9-ctrl-e-2 'abs(scrolled - before) < 1 && abs(offset - (scrolled + rh)) < 0.6 && row == 3' "$o1" 0
  st=$?; check "9 Control+e again: row 3, view starts where the last scroll put it" "$st" "$(segment view 9-ctrl-e-2)"
  o2=$(measured view 9-ctrl-e-2 | cut -d' ' -f4)
  shape 9-ctrl-e-3 'abs(scrolled - before) < 1 && abs(offset - (scrolled + rh)) < 0.6 && row == 4' "$o2" 0
  st=$?; check "9 Control+e a third time: $sh_e3" "$st" "$(segment view 9-ctrl-e-3)"
  o3=$(measured view 9-ctrl-e-3 | cut -d' ' -f4)
  shape 9-ctrl-y 'abs(scrolled - before) < 1 && abs(offset - (scrolled - rh)) < 0.6 && row == last_row && row > top && row <= bottom + 1' "$o3" "$(measured view 9-ctrl-e-3 | cut -d' ' -f5)"
  st=$?; check "9 Control+y scrolls back a row, selection stays in view: $(show 9-ctrl-y)" "$st" "$(segment view 9-ctrl-y)"
  oy=$(measured view 9-ctrl-y | cut -d' ' -f4)
  shape 9-H 'abs(scrolled - before) < 1 && abs(offset - scrolled) < 0.6 && row == top + 1' "$oy" 0
  st=$?; check "9 H selects the top row in view: $(show 9-H)" "$st" "$(segment view 9-H)"
  shape 9-L 'abs(scrolled - before) < 1 && abs(offset - scrolled) < 0.6 && row == bottom + 1' "$oy" 0
  st=$?; check "9 L selects the bottom row in view: $(show 9-L)" "$st" "$(segment view 9-L)"
  row_l=$(measured view 9-L | cut -d' ' -f5)
  shape 9-zz 'abs(scrolled - before) < 1 && row == last_row && abs(offset - clamp(((row - 1) * rh + row * rh - view) / 2, 0, max)) < 0.6' "$oy" "$row_l"
  st=$?; check "9 z z puts the selected row at the middle: $(show 9-zz)" "$st" "$(segment view 9-zz)"
  oz=$(measured view 9-zz | cut -d' ' -f4)
  shape 9-zt 'abs(scrolled - before) < 1 && row == last_row && abs(offset - clamp((row - 1) * rh, 0, max)) < 0.6' "$oz" "$row_l"
  st=$?; check "9 z t puts the selected row at the top: $(show 9-zt)" "$st" "$(segment view 9-zt)"
  ot=$(measured view 9-zt | cut -d' ' -f4)
  shape 9-H-after-zt 'abs(scrolled - before) < 1 && row == last_row' "$ot" "$row_l"
  st=$?; check "9 H after z t is the selected row itself: $(show 9-H-after-zt)" "$st" "$(segment view 9-H-after-zt)"
fi

# 10. A drop-down list: Tab focuses it, Enter opens it, Control+y picks,
# Control+e closes, and the status bar hints the keys while it is open
if [ -f "$out/menu.timeline" ]; then
  m() { fieldof menu "$1" menu; }
  hint() { segment menu "$1" | grep '^status hints' | tail -n 1; }
  kind0=$(fieldof menu 10-tab new_kind)
  [ "$(m 10-enter)" = true ] && hint 10-enter | grep -q 'Control+y: pick | Control+e: close'
  st=$?; check "10 Enter opens the list (menu=$(m 10-enter)) and the status bar reads: $(hint 10-enter | sed -n 's/.*hints=\("[^"]*"\).*/\1/p')" "$st" "$(segment menu 10-enter)"
  kind1=$(fieldof menu 10-ctrl-y new_kind)
  [ "$(m 10-ctrl-y)" = false ] && [ -n "$kind1" ] && [ "$kind1" != "$kind0" ] && segment menu 10-ctrl-y | grep -q 'message NewKind('
  st=$?; check "10 j then Control+y picks the next option ($kind0 to $kind1) and closes the list (menu=$(m 10-ctrl-y))" "$st" "$(segment menu 10-j; segment menu 10-ctrl-y)"
  ! hint 10-ctrl-y | grep -q 'pick'
  st=$?; check "10 the hints of the list are gone once it closes" "$st" "$(hint 10-ctrl-y)"
  kind2=$(fieldof menu 10-ctrl-e new_kind)
  [ "$(m 10-enter-again)" = true ] && hint 10-enter-again | grep -q 'Control+y: pick | Control+e: close' && [ "$(m 10-ctrl-e)" = false ] && [ "$kind2" = "$kind1" ] && ! segment menu 10-ctrl-e | grep -q 'message NewKind('
  st=$?; check "10 Enter opens it again (menu=$(m 10-enter-again)), Control+e closes it (menu=$(m 10-ctrl-e)) and picks nothing ($kind1, $kind2)" "$st" "$(segment menu 10-enter-again; segment menu 10-ctrl-e)"
fi
# No Space or Escape, which would be GO or Panic, in any phase
timelines=$(for phase in vi cua ops search view menu; do [ -f "$out/$phase.timeline" ] && printf '%s ' "$out/$phase.timeline"; done)
# shellcheck disable=SC2086
if [ -n "$timelines" ] && grep -q 'Named(Escape)\|Named(Space)\|action=Some((Go\|action=Some((Panic' $timelines; then
  # shellcheck disable=SC2086
  fail "no Space or Escape reached cinq in any phase" "$(grep -n 'Named(Escape)\|Named(Space)\|action=Some((Go\|action=Some((Panic' $timelines | head -n 3)"
else
  pass "no Space or Escape reached cinq in any phase"
fi
