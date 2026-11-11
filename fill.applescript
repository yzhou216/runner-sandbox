-- SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
--
-- SPDX-License-Identifier: 0BSD OR CC0-1.0

-- osascript fill.applescript PROCESS USER PASSWORD
--
-- Fills the name and password of the window of PROCESS that asks for
-- them and presses its button that allows, through accessibility.
on run argv
	set {owner, user, secret} to argv
	set seen to {}
	tell application "System Events" to tell process owner
		repeat with candidate in (every window)
			set fields to count of (every text field of candidate)
			set end of seen to (name of candidate as text) & ": " & fields & " text fields"
			if fields ≥ 2 then
				set value of text field 1 of candidate to user
				set value of text field 2 of candidate to secret
				delay 1
				set end of seen to "buttons " & (name of every button of candidate as text)
				repeat with wanted in {"OK", "Install Software", "Modify Settings", "Unlock"}
					if exists button (wanted as text) of candidate then
						click button (wanted as text) of candidate
						return "pressed " & wanted & " of " & owner
					end if
				end repeat
			end if
		end repeat
	end tell
	set AppleScript's text item delimiters to "; "
	error "no window of " & owner & " asks for a password: " & (seen as text)
end run
