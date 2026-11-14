# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Presses GO in the cinq that runs, COUNT times, PAUSE seconds apart, with a
# screenshot after each under LABEL
#
# Usage: go.ps1 LABEL [COUNT [PAUSE]]

param([string]$Label, [int]$Count = 1, [int]$Pause = 8)

. "$PSScriptRoot\ui.ps1"

$cinq = Get-Process cinq -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0 | Select-Object -First 1
if (-not $cinq) { Log 'no cinq window'; exit 1 }
foreach ($i in 1..$Count) {
    $null = [Input]::SetForegroundWindow($cinq.MainWindowHandle)
    Start-Sleep -Milliseconds 300
    Log "GO $Label $i"
    [System.Windows.Forms.SendKeys]::SendWait(' ')
    Start-Sleep $Pause
    Shot "$Label-$i"
}
