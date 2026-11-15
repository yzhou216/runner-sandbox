# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Provokes Windows Defender Firewall's prompt as a program that listens on a
# port does, with a copy of python.exe under a new name, and tries to answer
# it with keys, to learn whether the prompt shows on this Windows, which
# windows it adds, and what answers it.  Rules, windows, and screenshots go
# to out\.
#
# Usage: firewall.ps1

. "$PSScriptRoot\ui.ps1"

function Rules { Get-NetFirewallRule | Where-Object DisplayName -match 'probe' | ForEach-Object { "$($_.DisplayName) $($_.Direction) $($_.Action) $($_.Enabled)" } }
function Titles { [Windows32]::Titles() -join ' | ' }

$python = (Get-Command python).Source
$probe = Join-Path (Split-Path $python) 'probe.exe'
Copy-Item $python $probe
Log "titles before: $(Titles)"
Log "rules before: $((Rules) -join '; ')"
$listener = Start-Process $probe -ArgumentList '-c', '"import socket, time; s = socket.socket(); s.bind((chr(48) + chr(46) + chr(48) + chr(46) + chr(48) + chr(46) + chr(48), 5999)); s.listen(); time.sleep(120)"' -PassThru -WindowStyle Hidden
Start-Sleep 6
Log "titles with listener: $(Titles)"
Shot 'firewall-0-listening'
$tree = Windows
Log "UI Automation sees: $(($tree | Where-Object Kind -eq 'Window' | ForEach-Object Name) -join ' | ')"
Log "rules after listening: $((Rules) -join '; ')"

# Attempts to answer, each followed by a look
$attempts = @(
    @('tab to Allow and Enter', { [System.Windows.Forms.SendKeys]::SendWait('{TAB}'); Start-Sleep -Milliseconds 500; [System.Windows.Forms.SendKeys]::SendWait('{ENTER}') }),
    @('Alt+A', { [System.Windows.Forms.SendKeys]::SendWait('%a') }),
    @('Enter', { [System.Windows.Forms.SendKeys]::SendWait('{ENTER}') })
)
$n = 1
foreach ($attempt in $attempts) {
    $title = Titles
    if ($title -notmatch 'Windows Security') { Log "no prompt before attempt '$($attempt[0])'"; break }
    Log "attempt '$($attempt[0])'"
    & $attempt[1]
    Start-Sleep 3
    Shot "firewall-$n-$($attempt[0] -replace '\W', '')"
    Log "titles after: $(Titles)"
    Log "rules after: $((Rules) -join '; ')"
    $n++
}
Get-NetFirewallRule | Where-Object DisplayName -match 'probe|Python' |
    Get-NetFirewallApplicationFilter | Format-Table -AutoSize Program | Out-String | Set-Content "$out\firewall-probe-rules.txt"
Stop-Process -Id $listener.Id -Force -ErrorAction SilentlyContinue
