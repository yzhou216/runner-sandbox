# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Closes the "Windows Subsystem for Linux must be updated" window of
# windows-11-arm as soon as it opens, for SECONDS seconds, since it comes back
# every minute or so and takes the keys meant for cinq
# Usage: closer.ps1 SECONDS

. "$PSScriptRoot\ui.ps1"
$end = (Get-Date).AddSeconds([int]$args[0])
while ((Get-Date) -lt $end) {
    CloseWsl
    Start-Sleep -Milliseconds 250
}
