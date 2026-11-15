-- SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
--
-- SPDX-License-Identifier: 0BSD OR CC0-1.0

-- Presses Space COUNT times, GAP seconds apart, in the frontmost program,
-- and prints the time of day just before each press
on run argv
	set n to (item 1 of argv) as integer
	set gap to (item 2 of argv) as real
	set stamps to {}
	repeat n times
		set end of stamps to (tell current application to do shell script "perl -MTime::HiRes=time -e 'printf q(%.3f), time'")
		tell application "System Events" to keystroke " "
		delay gap
	end repeat
	set AppleScript's text item delimiters to " "
	return stamps as text
end run
