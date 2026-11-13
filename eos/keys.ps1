# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Types into Eos as an operator would on a PC keyboard, one step of STEPS
# per line: `keys TEXT` sends TEXT in the notation of SendKeys, `go` presses
# Space, which is GO, `wait SECONDS` waits, and `shot LABEL` takes a
# screenshot and writes down the windows.  Lines that start with # are
# comments.
#
# Usage: keys.ps1 STEPS

param([string]$Steps)

. "$PSScriptRoot\ui.ps1"

$eos = Get-Process Eos -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0 | Select-Object -First 1
if (-not $eos) { Log 'no Eos window'; exit 1 }
foreach ($line in Get-Content $Steps) {
    if (-not $line.Trim() -or $line.StartsWith('#')) { continue }
    $verb, $rest = $line -split ' ', 2
    switch ($verb) {
        'keys' {
            $null = [Input]::SetForegroundWindow($eos.MainWindowHandle)
            Start-Sleep -Milliseconds 300
            Log "keys $rest"
            [System.Windows.Forms.SendKeys]::SendWait($rest)
            Start-Sleep -Milliseconds 500
        }
        'go' {
            $null = [Input]::SetForegroundWindow($eos.MainWindowHandle)
            Log 'go'
            [System.Windows.Forms.SendKeys]::SendWait(' ')
        }
        'wait' { Start-Sleep -Seconds ([double]$rest) }
        'shot' {
            Shot "keys-$rest"
            (Windows | ForEach-Object Text) -join "`n" | Set-Content "$out\uia\keys-$rest.txt"
        }
    }
}
