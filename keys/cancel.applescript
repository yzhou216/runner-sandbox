-- SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
--
-- SPDX-License-Identifier: 0BSD OR CC0-1.0

-- Presses the Cancel button of the first window that has one, of any process,
-- and prints the process, or "none"
tell application "System Events"
	repeat with p in (every process)
		try
			click button "Cancel" of window 1 of p
			return name of p
		end try
	end repeat
	return "none"
end tell
