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

for phase in vi cua ops search line view menu outs file; do
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

# stray_keys FILE...: lines where Space, Escape, GO, or Panic reached cinq,
# apart from the dumps of the configuration and of the opened show, which
# list the bindings of the keymap in an order of their own
stray_keys() {
  grep -H -n 'Named(Escape)\|Named(Space)\|action=Some((Go\|action=Some((Panic' "$@" 2> /dev/null |
    grep -v ':\(update\|message\) \(Reconfigured\|Opened\)'
}

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
  stray=$(stray_keys "$out/vi.timeline" "$out/cua.timeline")
  if [ -n "$stray" ]; then
    fail "5 no Escape reached cinq" "$(printf '%s\n' "$stray" | head -n 3)"
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
  st=$?; check "7 the clipboard notice reads Cut 3 cues: $(segment ops 7-d2j | grep '^update' | tail -n 1 | sed -n 's/.*clipped=\(.*\) remark=.*$/\1/p')" "$st" "$told"
  segment ops 7-p | grep -q 'message Paste(' && [ "$pasted" = 30 ]
  st=$?; check "7 p pastes the three cues below: rows $cut to $pasted" "$st" "$(segment ops 7-p | grep '^press\|^message\|^update' | tail -n 4)"
  pending=$(fieldof ops 11-du keys)
  ! segment ops 11-du | grep -q 'message Undo' && [ "$stray" = "$pasted" ] && [ "$pending" = "[]" ]
  st=$?; check "11 d u does not undo: rows $pasted to $stray, keys left $pending" "$st" "$(segment ops 11-du | grep '^press\|^message\|^update' | tail -n 4)"
  typed=$(segment ops 11-2d | grep '^update' | tail -n 1)
  after=$(segment ops 11-ctrl-c | grep '^update' | tail -n 1)
  ! printf '%s\n' "$typed" | grep -q 'keys=\[\]' && segment ops 11-ctrl-c | grep -q "press Press { key: Char('c'), ctrl: true" && printf '%s\n' "$after" | grep -q 'keys=\[\]' &&
    ! segment ops 11-ctrl-c | grep -q 'message Cut\|message Delete\|message Copy\|across=Some' && [ "$(rows_of 11-ctrl-c)" = "$stray" ]
  st=$?; check "11 Control+c after 2 d drops the typed keys ($(printf '%s\n' "$typed" | sed -n 's/.* keys=\(\[[^]]*\]\).*/\1/p') to $(printf '%s\n' "$after" | sed -n 's/.* keys=\(\[[^]]*\]\).*/\1/p')) and cuts nothing: rows $stray to $(rows_of 11-ctrl-c)" "$st" "$(segment ops 11-2d | grep '^press\|^message\|^update' | tail -n 3; segment ops 11-ctrl-c | grep '^press\|^message\|^update' | tail -n 4)"
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
  # A wrap tells so in the status bar: n after ? goes up past the top and
  # N goes down past the bottom
  last_update() { segment search "$1" | grep '^update' | tail -n 1; }
  gg_row=$(row_of 8-gg-wrap); n_row=$(row_of 8-n-wrap); big_n_row=$(row_of 8-N-wrap)
  last_update 8-n-wrap | grep -q 'remark=Some("Search continued at the bottom")' && [ "$gg_row" = 1 ] && [ "$n_row" = 21 ]
  st=$?; check "8 n after ? wraps from row $gg_row to row $n_row with the notice: $(last_update 8-n-wrap | sed -n 's/.* remark=//p')" "$st" "$(segment search 8-n-wrap | grep '^press\|^message\|^update')"
  last_update 8-N-wrap | grep -q 'remark=Some("Search continued at the top")' && [ "$big_n_row" = 1 ]
  st=$?; check "8 N after ? wraps from row $n_row to row $big_n_row with the notice: $(last_update 8-N-wrap | sed -n 's/.* remark=//p')" "$st" "$(segment search 8-N-wrap | grep '^press\|^message\|^update')"
  last_update 8-n3 | grep -q 'remark=None' && last_update 8-N | grep -q 'remark=None'
  st=$?; check "8 n and N that do not wrap show no notice" "$st" "$(last_update 8-n3; last_update 8-N)"
  help=$(fieldof search h-f1 help)
  closed=$(fieldof search h-q help)
  [ "$help" = true ] && [ "$closed" = false ]
  st=$?; check "F1 shows the help page (help=$help), q closes it (help=$closed)" "$st" "$(segment search h-f1 | grep '^press\|^update'; segment search h-q | grep '^press\|^update')"
fi

# 12. The search line: ArrowUp recalls the newest earlier search that starts
# with the text typed, with the caret at the end, so that the next letter
# follows it, Control+u clears the line, and Control+c closes it and puts the
# selection back where the search began
if [ -f "$out/line.timeline" ]; then
  upd() { segment line "$1" | grep '^update' | tail -n 1; }
  row_of() { number selected_row "$(state line "$1")"; }
  text_of() { upd "$1" | sed -nE 's/.* search=(None|Some\(\("[^"]*", [a-z]*\)\)).*/\1/p'; }
  both() { [ "$(row_of "$1")" = "$2" ] && [ "$(text_of "$1")" = "$3" ]; }
  [ "$(row_of 12-gg)" = 1 ] && [ "$(row_of 12-enter-bl)" = 2 ] && [ "$(row_of 12-enter-br)" = 4 ] && [ "$(row_of 12-gg-2)" = 1 ] && both 12-b 2 'Some(("B", false))'
  st=$?; check "12 /Bl Enter, /Br Enter, gg, / B: rows $(for l in 12-gg 12-enter-bl 12-enter-br 12-gg-2 12-b; do printf '%s ' "$(row_of "$l")"; done)(1 2 4 1 2), text $(text_of 12-b)" "$st" "$(for l in 12-enter-bl 12-enter-br 12-b; do upd "$l"; done)"
  segment line 12-up | grep -q 'message SearchKey(Older)' && both 12-up 4 'Some(("Br", false))'
  st=$?; check "12 ArrowUp recalls the earlier search: text $(text_of 12-up), row $(row_of 12-up)" "$st" "$(segment line 12-up | grep '^press\|^message\|^update')"
  both 12-i 4 'Some(("Bri", false))'
  st=$?; check "12 the next letter follows the recalled text, so the caret is at its end: text $(text_of 12-i)" "$st" "$(segment line 12-i | grep '^press\|^message\|^update')"
  segment line 12-ctrl-u | grep -q 'message SearchKey(DeleteAll)' && both 12-ctrl-u 1 'Some(("", false))'
  st=$?; check "12 Control+u clears the line and the selection goes back: text $(text_of 12-ctrl-u), row $(row_of 12-ctrl-u)" "$st" "$(segment line 12-ctrl-u | grep '^press\|^message\|^update')"
  both 12-bo 6 'Some(("Bo", false))'
  st=$?; check "12 typing again selects the match: text $(text_of 12-bo), row $(row_of 12-bo)" "$st" "$(segment line 12-bo | grep '^press\|^message\|^update')"
  segment line 12-ctrl-c | grep -q 'action=Some((Cancel' && both 12-ctrl-c 1 None
  st=$?; check "12 Control+c closes the line (search $(text_of 12-ctrl-c)) and returns the selection to row $(row_of 12-ctrl-c)" "$st" "$(segment line 12-ctrl-c | grep '^press\|^message\|^update')"
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
        # A row is in view when more than half of it shows, as cinq counts it
        top = floor(scrolled / rh + 0.5)
        bottom = ceil((scrolled + view) / rh - 0.5) - 1
        if (bottom < top) bottom = top
        max = content - view
        exit !($test)
      }"
  }
  show() { measured view "$1" | awk '{ printf "view %s content %s scrolled %s -> offset %s row %s", $1, $2, $3, $4, $5 }'; }
  # 3. Three Control+f in a row move three pages: each page scrolls the rows
  # that its line tells, from where the one before left the list
  pages=$(segment view 3-ctrl-f | grep -c '^page WholeDown')
  moves=$(segment view 3-ctrl-f | grep '^page WholeDown' | tail -n 1 | sed -n 's/.* moves=\([0-9]*\).*/\1/p')
  chain=$(segment view 3-ctrl-f | grep '^measured' | sed -n 's/^measured view=[0-9.]* content=\([0-9.]*\) scrolled=\([0-9.-]*\) -> offset=\([0-9.-]*\) row=\([0-9]*\).*/\1 \2 \3 \4/p' |
    awk -v cues="$cues" -v moves="${moves:-0}" 'function abs(x) { return x < 0 ? -x : x }
      { rh = $1 / cues; n++; ok = ok && 1; if (n == 1 && abs($2) > 1) bad = 1; if (n > 1 && abs($2 - prev) > 1) bad = 1
        if (abs(($3 - $2) - moves * rh) > 1) bad = 1; prev = $3; last = $0 }
      END { printf "%d %s %s", n, bad ? "bad" : "good", last }')
  set -- $chain
  [ "$pages" = 3 ] && [ "$1" = 3 ] && [ "$2" = good ]
  st=$?; check "3 three Control+f in a row move three pages: $pages page lines of $moves rows, $1 landings, the last from $4 to $5 (row $6)" "$st" "$(segment view 3-ctrl-f | grep '^press\|^page\|^measured\|^update Landed')"
  sh_e1=$(show 9-ctrl-e-1); sh_e3=$(show 9-ctrl-e-3)
  shape 9-ctrl-e-1 'abs(offset - (scrolled + rh)) < 0.6 && row == 2' 0 0
  st=$?; check "9 Control+e scrolls a row, and selection moves onto the first row in view: $sh_e1" "$st" "$(segment view 9-ctrl-e-1 | grep '^press\|^measured\|^update Landed\|^update Reached')"
  o1=$(measured view 9-ctrl-e-1 | cut -d' ' -f4)
  shape 9-ctrl-e-2 'abs(scrolled - before) < 1 && abs(offset - (scrolled + rh)) < 0.6 && row == 3' "$o1" 0
  st=$?; check "9 Control+e again: row 3, view starts where the last scroll put it" "$st" "$(segment view 9-ctrl-e-2 | grep '^press\|^measured\|^update Landed\|^update Reached')"
  o2=$(measured view 9-ctrl-e-2 | cut -d' ' -f4)
  shape 9-ctrl-e-3 'abs(scrolled - before) < 1 && abs(offset - (scrolled + rh)) < 0.6 && row == 4' "$o2" 0
  st=$?; check "9 Control+e a third time: $sh_e3" "$st" "$(segment view 9-ctrl-e-3 | grep '^press\|^measured\|^update Landed\|^update Reached')"
  o3=$(measured view 9-ctrl-e-3 | cut -d' ' -f4)
  shape 9-ctrl-y 'abs(scrolled - before) < 1 && abs(offset - (scrolled - rh)) < 0.6 && row == last_row && row > top && row <= bottom + 1' "$o3" "$(measured view 9-ctrl-e-3 | cut -d' ' -f5)"
  st=$?; check "9 Control+y scrolls back a row, selection stays in view: $(show 9-ctrl-y)" "$st" "$(segment view 9-ctrl-y | grep '^press\|^measured\|^update Landed\|^update Reached')"
  oy=$(measured view 9-ctrl-y | cut -d' ' -f4)
  shape 9-H 'abs(scrolled - before) < 1 && abs(offset - scrolled) < 0.6 && row == top + 1' "$oy" 0
  st=$?; check "9 H selects the top row in view: $(show 9-H)" "$st" "$(segment view 9-H | grep '^press\|^measured\|^update Landed\|^update Reached')"
  shape 9-L 'abs(scrolled - before) < 1 && abs(offset - scrolled) < 0.6 && row == bottom + 1' "$oy" 0
  st=$?; check "9 L selects the bottom row in view: $(show 9-L)" "$st" "$(segment view 9-L | grep '^press\|^measured\|^update Landed\|^update Reached')"
  row_l=$(measured view 9-L | cut -d' ' -f5)
  shape 9-zz 'abs(scrolled - before) < 1 && row == last_row && abs(offset - clamp(((row - 1) * rh + row * rh - view) / 2, 0, max)) < 0.6' "$oy" "$row_l"
  st=$?; check "9 z z puts the selected row at the middle: $(show 9-zz)" "$st" "$(segment view 9-zz | grep '^press\|^measured\|^update Landed\|^update Reached')"
  oz=$(measured view 9-zz | cut -d' ' -f4)
  shape 9-zt 'abs(scrolled - before) < 1 && row == last_row && abs(offset - clamp((row - 1) * rh, 0, max)) < 0.6' "$oz" "$row_l"
  st=$?; check "9 z t puts the selected row at the top: $(show 9-zt)" "$st" "$(segment view 9-zt | grep '^press\|^measured\|^update Landed\|^update Reached')"
  ot=$(measured view 9-zt | cut -d' ' -f4)
  shape 9-H-after-zt 'abs(scrolled - before) < 1 && row == last_row' "$ot" "$row_l"
  st=$?; check "9 H after z t is the selected row itself: $(show 9-H-after-zt)" "$st" "$(segment view 9-H-after-zt | grep '^press\|^measured\|^update Landed\|^update Reached')"
  # A far move to a row lands it in the middle of the view
  far=$(segment view 9-55G | grep '^revealed' | tail -n 1)
  far_view=$(printf '%s\n' "$far" | sed -n 's/^revealed view=\([0-9.]*\) .*/\1/p')
  far_top=$(printf '%s\n' "$far" | sed -n 's/.* row_top=\([0-9.]*\) .*/\1/p')
  far_height=$(printf '%s\n' "$far" | sed -n 's/.* row_height=\([0-9.]*\) .*/\1/p')
  far_offset=$(printf '%s\n' "$far" | sed -n 's/.*-> offset=Some(\([0-9.-]*\)).*/\1/p')
  far_seen=$(segment view 9-55G | grep '^measure ' | tail -n 1 | sed -n 's/.* scrolled=\([0-9.-]*\).*/\1/p')
  far_row=$(number selected_row "$(state view 9-55G)")
  awk -v v="${far_view:-0}" -v t="${far_top:-0}" -v h="${far_height:-0}" -v o="${far_offset:-x}" -v seen="${far_seen:-x}" 'function abs(x) { return x < 0 ? -x : x }
    BEGIN { middle = t + h / 2 - v / 2; exit !(o != "x" && seen != "x" && abs(o - middle) < 1 && abs(seen - o) < 2) }'
  st=$?; [ "$far_row" = 55 ] || st=1
  check "9 55 G selects row $far_row and centers it: scrolls to $far_offset (middle $(awk -v v="${far_view:-0}" -v t="${far_top:-0}" -v h="${far_height:-0}" 'BEGIN { print t + h / 2 - v / 2 }')), measured $far_seen" "$st" "$(segment view 9-55G | grep '^press\|^revealed\|^measure \|^update')"
fi

# 10. A drop-down list: Tab focuses it, Enter opens it, Control+y picks,
# Control+e closes, and the status bar hints the keys while it is open
if [ -f "$out/menu.timeline" ]; then
  m() { fieldof menu "$1" menu; }
  hint() { segment menu "$1" | grep '^status hints' | tail -n 1; }
  kind0=$(fieldof menu 10-tab-9 new_kind)
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
# focusline PHASE LABEL: the last line "focus ..." that the step LABEL caused
focusline() { segment "$1" "$2" | grep '^focus backward' | tail -n 1; }
# widgetline PHASE LABEL: the last line "focused widget ..." of the step
widgetline() { segment "$1" "$2" | grep '^focused widget' | tail -n 1; }
# order_of LINE: the place in the order of the window of the widget focused
order_of() { printf '%s\n' "$1" | sed -n 's/^focused widget order=\([0-9]*\).*/\1/p'; }
# first_inspector LINE: the place of the first field of the inspector among
# the fields that a "focus" line lists
first_inspector() {
  printf '%s\n' "$1" | sed -n 's/^focus backward=[a-z]* slots=\[\(.*\)\] -> target.*/\1/p' |
    grep -o '([a-z]*, [a-z]*)' | grep -n '^(true' | head -n 1 | cut -d: -f1 | awk '{ print $1 - 1 }'
}

# 13. Tab from the cue list reaches the first field of the inspector first
if [ -f "$out/menu.timeline" ] && has_step menu 13-tab-1; then
  f1=$(focusline menu 13-tab-1); w1=$(widgetline menu 13-tab-1)
  want=$(first_inspector "$f1")
  printf '%s\n' "$f1" | grep -q 'in_inspector=Some(true)' && ! printf '%s\n' "$f1" | sed -n 's/.*slots=\[\(.*\)\] ->.*/\1/p' | grep -q 'true)' &&
    [ -n "$want" ] && [ "$(order_of "$w1")" = "$want" ]
  st=$?; check "13 the first Tab from the cue list focuses the widget at place $(order_of "$w1"), the first field of the inspector (place ${want:-none}): $(printf '%s\n' "$w1" | sed -n 's/^focused widget //p')" "$st" "$f1 / $w1"
  w2=$(widgetline menu 13-tab-2)
  printf '%s\n' "$w2" | grep -q 'id=Some(Id(Custom("name")))' && [ "$(order_of "$w2")" = "$((want + 1))" ]
  st=$?; check "13 the second Tab reaches the Name field, the next in the inspector: $(printf '%s\n' "$w2" | sed -n 's/^focused widget //p')" "$st" "$(focusline menu 13-tab-2) / $w2"
  w10=$(widgetline menu 10-tab-9)
  [ "$(order_of "$w10")" = 0 ]
  st=$?; check "13 the toolbar's list comes after the fields of the inspector, at the ninth Tab: $(printf '%s\n' "$w10" | sed -n 's/^focused widget //p')" "$st" "$(focusline menu 10-tab-9) / $w10"
fi

# 14. Enter in the Name or Channels field of an output applies the text and
# leaves the field, so that the next key acts on the outputs
if [ -f "$out/outs.timeline" ]; then
  o() { segment outs "$1" | grep '^update' | tail -n 1; }
  names() { o "$1" | sed -n 's/.* outputs=\[\(.*\)\]$/\1/p'; }
  out_of() { o "$1" | sed -En 's/.* output=(Some\(OutputId\([0-9]*\)\)|None) .*/\1/p'; }
  o 14-l | grep -q ' shown=true ' && [ "$(out_of 14-l)" = 'Some(OutputId(1))' ]
  st=$?; check "14 l shows the outputs and selects the first: $(out_of 14-l)" "$st" "$(o 14-l)"
  segment outs 14-cc | grep -q 'message Focus(Name(' && segment outs 14-name | grep -q 'message SettingTyped(OutputName(OutputId(1)), "Front")' &&
    names 14-name | grep -q '^OutputId(1)=Front@1,'
  st=$?; check "14 c c puts the keyboard in the Name field and Front replaces the name: $(names 14-name)" "$st" "$(segment outs 14-cc | grep '^message\|^update' | tail -n 3; segment outs 14-name | grep '^message' | tail -n 2)"
  segment outs 14-enter | grep -q '^message Focus(Cues)' && names 14-enter | grep -q '^OutputId(1)=Front@1,' && ! segment outs 14-enter | grep -q 'SettingTyped'
  st=$?; check "14 Enter in the Name field applies Front and leaves the field (Focus(Cues)): $(names 14-enter)" "$st" "$(segment outs 14-enter | grep '^message\|^update')"
  segment outs 14-j | grep -q 'action=Some((Next' && ! segment outs 14-j | grep -q 'SettingTyped' && [ "$(out_of 14-j)" = 'Some(OutputId(2))' ] &&
    names 14-j | grep -q '^OutputId(1)=Front@1,'
  st=$?; check "14 j after Enter moves to the next output ($(out_of 14-j)) and types nothing: $(names 14-j)" "$st" "$(segment outs 14-j | grep '^press\|^message\|^update')"
  t1=$(widgetline outs 14-tab-1); t3=$(widgetline outs 14-tab-3)
  printf '%s\n' "$t1" | grep -q 'id=Some(Id(Custom("output name")))' && [ "$(order_of "$t3")" = 3 ]
  st=$?; check "14 Tab, Tab, Tab from the outputs reach the Channels field (place $(order_of "$t3"), after the Name field and the Device list)" "$st" "$t1 / $t3"
  segment outs 14-channels | grep -q 'SettingTyped(.*"1-2")' && names 14-channels | grep -q 'OutputId(2)=Right@2,'
  st=$?; check "14 1-2 typed into the Channels field waits for Enter: $(names 14-channels)" "$st" "$(segment outs 14-channels | grep '^message\|^update' | tail -n 3)"
  segment outs 14-enter-channels | grep -q '^message Focus(Cues)' && names 14-enter-channels | grep -q 'OutputId(2)=Right@1-2,'
  st=$?; check "14 Enter in the Channels field applies 1-2 and leaves the field: $(names 14-enter-channels)" "$st" "$(segment outs 14-enter-channels | grep '^message\|^update')"
  segment outs 14-k | grep -q 'action=Some((Previous' && ! segment outs 14-k | grep -q 'SettingTyped' && [ "$(out_of 14-k)" = 'Some(OutputId(1))' ] &&
    names 14-k | grep -q 'OutputId(2)=Right@1-2,'
  st=$?; check "14 k after Enter moves to the previous output ($(out_of 14-k)) and types nothing" "$st" "$(segment outs 14-k | grep '^press\|^message\|^update')"
fi

# 15. The help page joins the keys of a sequence of plain letters and opens
# with its first steps
if [ -f "$out/search.timeline" ] && has_step search h-f1; then
  help=$(segment search h-f1 | grep '^helptext' | tail -n 1)
  printf '%s\n' "$help" | grep -q 'Cut = dd,' && printf '%s\n' "$help" | grep -q 'Copy = yy,' && printf '%s\n' "$help" | grep -q 'First = gg,' &&
    ! printf '%s\n' "$help" | grep -q 'Cut = d d\|First = g g'
  st=$?; check "15 the help page lists sequences of letters joined: $(printf '%s\n' "$help" | grep -o '\(Cut\|Copy\|First\|ChangeName\) = [^;]*' | tr '\n' ';')" "$st" "$help"
  printf '%s\n' "$help" | grep -q 'first_steps="First steps: .* cc names it, and u undoes the last change\.'
  st=$?; check "15 the first steps paragraph: $(printf '%s\n' "$help" | sed -n 's/.*first_steps="\([^"]*\)".*/\1/p' | cut -c1-120)..." "$st" "$help"
fi

# 16. Enter in an empty File field opens the file chooser
if [ -f "$out/file.timeline" ]; then
  w3=$(widgetline file 16-tab-3)
  printf '%s\n' "$w3" | grep -q 'id=Some(Id(Custom("file")))' && [ "$(order_of "$w3")" = 4 ]
  st=$?; check "16 the third Tab reaches the File field of the audio cue: $(printf '%s\n' "$w3" | sed -n 's/^focused widget //p')" "$st" "$(focusline file 16-tab-3) / $w3"
  # What the status bar read from the moment the field had the keyboard until
  # Enter, which is where the hint of Enter belongs
  hint=$(awk '/^=== 16-enter$/ { exit } /^status hints/' "$out/file.timeline" | tail -n 1)
  printf '%s\n' "$hint" | grep -q 'Enter: choose a file'
  st=$?; check "16 the status bar hints Enter once the File field has the keyboard: $(printf '%s\n' "$hint" | sed -n 's/.*hints=\("[^"]*"\).*/\1/p')" "$st" "$(segment file 16-tab-3 | grep '^message\|^status')"
  segment file 16-enter | grep -q '^message ChooseFile(CueId(2))'
  st=$?; check "16 Enter in the empty File field sends ChooseFile: $(segment file 16-enter | grep '^message ChooseFile')" "$st" "$(segment file 16-enter | grep '^press\|^message')"
  if [ -f "$out/file-16-windows.txt" ]; then
    grep -q 'Choose sound file' "$out/file-16-windows.txt" && ! grep -q 'Choose sound file' "$out/file-16-windows-after.txt"
    st=$?; check "16 the file chooser showed after Enter and is gone after the cancel" "$st" "$(cat "$out/file-16-windows.txt" "$out/file-16-windows-after.txt" 2>&1)"
  fi
fi
# No Space or Escape, which would be GO or Panic, in any phase
timelines=$(for phase in vi cua ops search line view menu outs file; do [ -f "$out/$phase.timeline" ] && printf '%s ' "$out/$phase.timeline"; done)
# shellcheck disable=SC2086
stray=$(stray_keys $timelines)
if [ -n "$timelines" ] && [ -n "$stray" ]; then
  fail "no Space or Escape reached cinq in any phase" "$(printf '%s\n' "$stray" | head -n 3)"
else
  pass "no Space or Escape reached cinq in any phase"
fi
