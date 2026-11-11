-- SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
--
-- SPDX-License-Identifier: 0BSD OR CC0-1.0

-- osascript fill.applescript PROCESS USER PASSWORD
--
-- Fills the name and password of the window of PROCESS that asks for
-- them and presses its button that allows, through accessibility.
on run argv
	set {owner, user, secret} to argv
	tell application "System Events" to tell process owner
		repeat with dialog in windows
			if exists text field 2 of dialog then
				set value of text field 1 of dialog to user
				set value of text field 2 of dialog to secret
				delay 1
				repeat with label in {"OK", "Install Software", "Modify Settings", "Unlock"}
					if exists button label of dialog then
						click button label of dialog
						return "pressed " & label & " of " & owner
					end if
				end repeat
			end if
		end repeat
	end tell
	error "no dialog of " & owner & " asks for a password"
end run
