# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Provokes Windows Defender Firewall's prompt as a program that listens on a
# port does, with a copy of python.exe under a new name, and tries to answer
# it with keys and the mouse, to learn whether the prompt shows on this
# Windows, which rules it adds, and what answers it.  Rules, window titles,
# and screenshots go to out\.
#
# Usage: firewall.ps1

. "$PSScriptRoot\ui.ps1"

# Rules of the copy, with what each does
function Rules {
    Get-NetFirewallApplicationFilter | Where-Object Program -match 'probe\.exe' | ForEach-Object {
        $r = Get-NetFirewallRule -AssociatedNetFirewallApplicationFilter $_
        "$($r.DisplayName) $($r.Direction) $($r.Action) enabled=$($r.Enabled) profile=$($r.Profile)"
    }
}
function Titles { [Windows32]::Titles() -join ' | ' }
function Prompt { (Titles) -match 'Windows Security' }

$python = (Get-Command python).Source
$probe = Join-Path (Split-Path $python) 'probe.exe'
Copy-Item $python $probe
Log "titles before: $(Titles)"
$listener = Start-Process $probe -PassThru -WindowStyle Hidden -ArgumentList '-c',
    '"import socket, time; s = socket.socket(); s.bind((chr(48), 5999)); s.listen(); time.sleep(150)"'
Start-Sleep 6
Log "titles with the listener: $(Titles)"
Log "rules of the copy: $((Rules) -join '; ')"
CloseWsl
Shot 'firewall-0-listening'
Log "UI Automation sees: $((Windows | Where-Object Kind -eq 'Window' | ForEach-Object Name) -join ' | ')"

# Ways to answer, each followed by a look, until the prompt is gone
$send = { param($keys) [System.Windows.Forms.SendKeys]::SendWait($keys) }
$attempts = @(
    @('Enter', { & $send '{ENTER}' }),
    @('Tab then Enter', { & $send '{TAB}'; Start-Sleep -Milliseconds 500; & $send '{ENTER}' }),
    @('Alt+A', { & $send '%a' }),
    @('click Allow', { [Input]::Click(414, 527) })
)
$n = 1
foreach ($attempt in $attempts) {
    if (-not (Prompt)) { Log "no prompt before '$($attempt[0])'"; break }
    CloseWsl
    $null = [Windows32]::Focus('Windows Security')
    Start-Sleep -Milliseconds 500
    Log "attempt '$($attempt[0])'"
    & $attempt[1]
    Start-Sleep 3
    Shot ("firewall-$n-" + ($attempt[0] -replace '\W', ''))
    Log "prompt still there: $(Prompt); rules of the copy: $((Rules) -join '; ')"
    $n++
}
Log "titles at the end: $(Titles)"
Stop-Process -Id $listener.Id -Force -ErrorAction SilentlyContinue
