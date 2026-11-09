-- SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
--
-- SPDX-License-Identifier: 0BSD OR CC0-1.0

-- Prints the windows of the process named by the first argument, and in each
-- one a line per button, row, text field, and static text: class, name,
-- value, whether it is enabled, and whether it is selected
on run argv
	set procName to item 1 of argv
	set report to ""
	tell application "System Events"
		tell process procName
			repeat with w in windows
				set wName to ""
				try
					set wName to name of w as text
				end try
				set report to report & "WINDOW [" & wName & "]" & linefeed
				set found to {}
				try
					set found to entire contents of w
				end try
				repeat with e in found
					try
						set c to class of e as text
						if c is in {"button", "row", "text field", "static text", "pop up button", "radio button", "outline"} then
							set n to ""
							set v to ""
							set en to ""
							set sel to ""
							try
								set n to name of e as text
							end try
							try
								set v to value of e as text
							end try
							try
								set en to enabled of e as text
							end try
							try
								set sel to selected of e as text
							end try
							set report to report & "  " & c & " | " & n & " | " & v & " | enabled=" & en & " | selected=" & sel & linefeed
						end if
					end try
				end repeat
			end repeat
		end tell
	end tell
	return report
end run
