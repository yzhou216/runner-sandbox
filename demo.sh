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
  prompts=$((${prompts:-0} + 1))
  # Name and password through accessibility, in coreautha, which owns
  # the dialog: it ignores synthetic clicks, and the password field took
  # synthetic keys as some other password
  ./ui windows > "shots/prompt-$prompts-windows.txt"
  pressed=
  while IFS= read -r owner; do
    if osascript fill.applescript "$owner" "$user" "$password" 2>> "shots/prompt-$prompts-contents.txt"; then
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

# ffmpeg, which ends on SIGINT, where screencapture -V ran on past its limit
# while the screen stood still; Matroska stays readable if it is cut off
ffmpeg -loglevel error -f avfoundation -capture_cursor 1 -framerate 30 -i "Capture screen 0:none" \
  -c:v libx264 -preset ultrafast -pix_fmt yuv420p shots/demo.mkv < /dev/null &
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

caption "Privacy & Security offers Open Anyway, behind an administrator's password"
open "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension"
sleep 6
./ui scroll 620 400 -40
check ./ui wait "Open Anyway" 15
sleep 6

# The password dialog of Open Anyway, which coreautha shows, refused every
# password that a script gave it, through keys or accessibility
caption "This runner cannot answer that dialog, so Terminal clears the quarantine as Open Anyway would"
pkill -x "System Settings" || true
open -a Terminal
sleep 4
osascript -e 'tell application "System Events" to keystroke "xattr -d com.apple.quarantine ~/Downloads/'"$package"' && open ~/Downloads/'"$package"'"' \
  -e 'tell application "System Events" to key code 36'

caption "Installer opens"
check ./ui click Continue 30
sleep 3
caption "Install, for all users of this Mac"
./ui click Continue 6 || true
sleep 2
check ./ui click Install 20
if ./ui wait --contains "to allow" 6; then
  caption "Installer asks for the password"
  password_prompt
fi
caption "The installation succeeds"
check ./ui wait --contains successful 90
sleep 4
log show --last 3m --style compact --predicate 'process == "authd"' > shots/installer-authd.txt 2>&1 || true
check ./ui click Close
./ui click Keep 6 || true
sleep 2

caption "cinq is in Applications"
open /Applications
sleep 4
# Vision has read the name as "cing"; "cin" alone matched "Macintosh HD"
if ! check ./ui click --double cinq 10; then
  ./ui click --double cing 5 || open /Applications/cinq.app
fi
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
sleep 1

caption "The tutorial and its demo, in /usr/local/share/doc/cinq"
osascript -e 'tell application "System Events" to keystroke "clear; ls -l /usr/local/share/doc/cinq /usr/local/share/doc/cinq/demo"' \
  -e 'tell application "System Events" to key code 36'
sleep 6
caption "The tutorial opens in Preview"
open /usr/local/share/doc/cinq/tutorial.pdf
sleep 10
pkill -x Preview || true

# The command, since cinq.app takes no documents from Finder
caption "As the tutorial says, a copy of the demo, whose storm show cinq opens"
pkill -x cinq || true
open -a Terminal
sleep 2
osascript -e 'tell application "System Events" to keystroke "clear; cp -R /usr/local/share/doc/cinq/demo ~/demo && cinq ~/demo/storm.cinq.toml &"' \
  -e 'tell application "System Events" to key code 36'
sleep 12
caption "End"
sleep 3
end=$(now)
echo "recorded $(echo "$end - $start" | bc) seconds"

screencapture -x shots/last.png
stopped=$(now)
kill -INT "$recorder"
wait "$recorder"
# Seconds of video before the captions' clock started, which compose.sh
# skips
length=$(ffprobe -v error -show_entries format=duration -of csv=p=0 shots/demo.mkv)
echo "$length - ($stopped - $start)" | bc > shots/lead.txt
echo "lead $(cat shots/lead.txt) of $length seconds"
echo "failures: $failures"
