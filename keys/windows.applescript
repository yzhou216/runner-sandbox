-- SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
--
-- SPDX-License-Identifier: 0BSD OR CC0-1.0

-- Prints each process that has windows, with their names, one process to a
-- line.  The file dialog of cinq belongs to a service of its own, not to cinq.
tell application "System Events"
	set found to ""
	repeat with p in (every process)
		try
			set names to name of every window of p
			if (count of names) > 0 then
				set found to found & (name of p) & ": " & (names as text) & linefeed
			end if
		end try
	end repeat
	return found
end tell
