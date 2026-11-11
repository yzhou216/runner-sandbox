  # OK through accessibility, since SecurityAgent ignored synthetic clicks
  osascript -e 'tell application "System Events" to tell process "SecurityAgent" to get entire contents of window 1' > "shots/prompt-$prompts-contents.txt" 2>&1 || true
  osascript -e 'tell application "System Events" to tell process "SecurityAgent" to click button "OK" of window 1' || ./ui click OK 3 || ./ui key return
  sleep 3
  screencapture -x "shots/prompt-$prompts-after.png"
  log show --last 1m --style compact --predicate 'process == "authd" OR process == "SecurityAgent"' > "shots/prompt-$prompts-log.txt" 2>&1 || true
#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Installs cinq's package as a user would, from a download in Safari to
# the cinq command in Terminal, while screencapture records the screen.
# Each step's caption goes into shots/captions.tsv with its time since the
# recording started, and a screenshot of each failure into shots.
set -u
package=cinq-0.1.3-aarch64-darwin.pkg
url=https://github.com/yzhou216/runner-sandbox/raw/cinq/2026-11-11-darwin-pkg/$package
# Account that the workflow found or made
: "${user:?}" "${password:?}"
mkdir -p shots

now() { perl -MTime::HiRes=time -e 'printf "%.2f\n", time'; }
caption() {
  printf '%s\t%s\n' "$(echo "$(now) - $start" | bc)" "$1" >> shots/captions.tsv
  echo "== $1"
}
failures=0
check() {
  "$@" && return 0
  failures=$((failures + 1))
  echo "failed: $*"
  screencapture -x "shots/failure-$failures.png"
  ./ui list > "shots/failure-$failures.txt"
  return 1
}
password_prompt() {
  check ./ui wait --contains "to allow" 20 || return 1
  sleep 2
  if [ "$user" != runner ]; then
    ./ui key --shift tab
    ./ui key --command a
    ./ui keys "$user"
    ./ui key tab
  fi
  ./ui keys "$password"
  sleep 1
  prompts=$((${prompts:-0} + 1))
  screencapture -x "shots/prompt-$prompts.png"
  # OK through accessibility on whichever process owns the dialog, since
  # the dialog ignored synthetic clicks
  ./ui windows > "shots/prompt-$prompts-windows.txt"
  pressed=
  while IFS= read -r owner; do
    osascript -e "tell application \"System Events\" to tell process \"$owner\" to get entire contents of window 1" \
      >> "shots/prompt-$prompts-contents.txt" 2>&1 || true
    if osascript -e "tell application \"System Events\" to tell process \"$owner\" to click button \"OK\" of window 1" \
      >> "shots/prompt-$prompts-contents.txt" 2>&1; then
      echo "pressed OK of $owner"
      pressed=$owner
      break
    fi
  done < <(cut -f1 "shots/prompt-$prompts-windows.txt" | sort -u | grep -v -x -e Dock -e "Window Server" -e Finder)
  [ -n "$pressed" ] || ./ui click OK 3 || ./ui key return
  sleep 3
  screencapture -x "shots/prompt-$prompts-after.png"
  log show --last 1m --style compact \
    --predicate 'process IN {"authd", "SecurityAgent", "coreauthd", "coreautha", "opendirectoryd"}' \
    > "shots/prompt-$prompts-log.txt" 2>&1 || true
}

screencapture -x -V 360 shots/demo.mov &
recorder=$!
start=$(now)
sleep 3

caption "Safari downloads cinq's package from GitHub"
open -a Safari "$url"
./ui click --contains Allow 60 || true
for _ in $(seq 60); do [ -f ~/Downloads/$package ] && break; sleep 1; done
sleep 3
xattr -p com.apple.quarantine ~/Downloads/$package
# Not osascript, which waits two minutes for a Safari with a sheet open
pkill -x Safari || true
sleep 2

caption "A double click on the package in Downloads"
open ~/Downloads
sleep 4
if ! check ./ui click --double --contains cinq-0.1.3 15; then
  open ~/Downloads/$package &
fi
sleep 1

caption "Gatekeeper blocks it, since it is unsigned"
check ./ui wait Done 20
sleep 4
check ./ui click Done

caption "Privacy & Security offers Open Anyway"
open "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension"
sleep 6
./ui scroll 620 400 -40
sleep 3
check ./ui click "Open Anyway" 15
sleep 2

caption "Darwin asks for the password, then once more to open it"
for _ in 1 2 3; do
  sleep 2
  if ./ui wait Continue 3; then
    break
  elif ./ui wait --contains "to allow" 3; then
    password_prompt
  elif ./ui wait "Open Anyway" 3; then
    ./ui click "Open Anyway"
  fi
done

caption "Installer opens"
sleep 2
check ./ui click Continue 30
sleep 3
caption "Install, for all users of this Mac"
./ui click Continue 6 || true
sleep 2
check ./ui click Install 20
caption "Installer asks for the password"
password_prompt
caption "The installation succeeds"
check ./ui wait --contains successful 90
sleep 4
check ./ui click Close
./ui click Keep 10 || true
sleep 2

caption "cinq is in Applications"
open /Applications
sleep 4
check ./ui click --double cinq 15
caption "cinq opens with no prompt"
sleep 10

caption "The cinq command, its manual pages, and the receipts"
open -a Terminal
sleep 4
osascript -e 'tell application "System Events" to tell process "Terminal" to set position of window 1 to {40, 60}' || true
osascript -e 'tell application "System Events" to keystroke "clear; command -v cinq; cinq --version; ls -l /usr/local/bin/cinq /usr/local/share/man/man*/cinq*; pkgutil --pkgs | grep cinq"' -e 'tell application "System Events" to key code 36'
sleep 8
osascript -e 'tell application "System Events" to keystroke "man cinq"' -e 'tell application "System Events" to key code 36'
sleep 8
osascript -e 'tell application "System Events" to keystroke "q"'
caption "End"
sleep 3
end=$(now)
echo "recorded $(echo "$end - $start" | bc) seconds"

screencapture -x shots/last.png
wait "$recorder"
echo "failures: $failures"
