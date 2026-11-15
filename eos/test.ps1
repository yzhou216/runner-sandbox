# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs cinq's light cues (lights.toml) with listen.py beside it on the same
# computer, then its network cues (net.toml) against fakeeos.py, pressing GO
# with SendKeys as a user does.  Every step is marked with the time in
# seconds since the epoch, which lines up with the packets that listen.py
# saw and the messages that fakeeos.py logged.  The Windows firewall stays
# on, to see whether it asks anything.  cinq.exe comes from
# cinq-windows\bin in the checkout.
#
# Usage: test.ps1

. "$PSScriptRoot\ui.ps1"

$exe = "$root\cinq-windows\bin\cinq.exe"
$env:ICED_BACKEND = 'tiny-skia'

# What the computer is
Mark "computer $env:COMPUTERNAME arch $env:PROCESSOR_ARCHITECTURE"
(Get-CimInstance Win32_OperatingSystem | ForEach-Object { "$($_.Caption) $($_.Version) $($_.OSArchitecture)" }) | Set-Content "$out\os.txt"
python -c "import platform, sys; print(platform.machine(), platform.python_version(), sys.executable)" | Set-Content "$out\python.txt"
$pe = [IO.File]::ReadAllBytes($exe)
$machine = [BitConverter]::ToUInt16($pe, [BitConverter]::ToInt32($pe, 0x3C) + 4)
Add-Content "$out\os.txt" ('cinq.exe PE machine 0x{0:X4}' -f $machine)
Get-NetIPAddress -AddressFamily IPv4 | Format-Table -AutoSize InterfaceAlias, InterfaceIndex, IPAddress, PrefixLength | Out-String | Set-Content "$out\network.txt"
Get-NetAdapter | Format-Table -AutoSize Name, InterfaceDescription, Status, LinkSpeed | Out-String | Add-Content "$out\network.txt"
ipconfig /all | Add-Content "$out\network.txt"
route print -4 | Add-Content "$out\network.txt"
Get-NetFirewallProfile | Format-Table -AutoSize Name, Enabled, DefaultInboundAction, DefaultOutboundAction | Out-String | Set-Content "$out\firewall.txt"
$rulesBefore = Get-NetFirewallRule | ForEach-Object DisplayName | Sort-Object -Unique

# Python listens in this test, which brings up the firewall's prompt on
# windows-11-arm and takes the keys meant for cinq (firewall.ps1 looks at
# that prompt on its own), so allow it first.  cinq itself only sends, and
# any prompt of its own shows in the screenshots.
New-NetFirewallRule -DisplayName 'sandbox python' -Direction Inbound -Action Allow -Program (Get-Command python).Source -Profile Any | Out-Null

function WindowsNow([string]$label) {
    (Windows | ForEach-Object Text) -join "`n" | Set-Content "$out\uia\windows-$label.txt"
    Shot "windows-$label"
}

# Presses GO in the window of cinq, after closing the window of the
# subsystem for Linux, which would take the key
function Go([string]$label) {
    CloseWsl
    $cinq = Get-Process cinq -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0 | Select-Object -First 1
    if (-not $cinq) { Log 'no cinq window'; return }
    $null = [Input]::SetForegroundWindow($cinq.MainWindowHandle)
    Start-Sleep -Milliseconds 300
    Mark "GO $label"
    [System.Windows.Forms.SendKeys]::SendWait(' ')
}

function Start-Cinq([string]$show, [string]$name) {
    CloseWsl
    Mark "start cinq $show"
    $script:cinq = Start-Process $exe -ArgumentList "`"$root\eos\$show`"" -PassThru `
        -RedirectStandardOutput "$out\cinq-$name-out.txt" -RedirectStandardError "$out\cinq-$name-err.txt"
    $window = $null
    for ($i = 0; $i -lt 60 -and -not $window; $i++) {
        Start-Sleep 1
        $window = Get-Process -Id $script:cinq.Id -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0
    }
    if (-not $window) { Log 'no cinq window'; Shot "$name-none"; exit 1 }
    Mark "cinq window up after $i s"
    Start-Sleep 3
    Shot "$name-start"
}

# Size of a file that another process holds open, from the handle, since
# its directory entry lags
function Size([string]$path) {
    $f = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite,Delete')
    try { $f.Length } finally { $f.Dispose() }
}

# Watches the standard error of cinq for the first line after GO and logs
# when it came, and what it said
function Warned([string]$label, [string]$err, [double]$seconds = 5) {
    $before = Size $err
    $go = Epoch
    $end = (Get-Date).AddSeconds($seconds)
    while ((Get-Date) -lt $end) {
        if ((Size $err) -gt $before) {
            Mark ('warning for {0} after {1:F3} s of the GO' -f $label, ((Epoch) - $go))
            return
        }
        Start-Sleep -Milliseconds 20
    }
    Mark "no warning for $label within $seconds s"
}

# Phase 1: light cues, seen by a listener on this computer
Mark 'phase lights'
Start-Cinq 'lights.toml' 'lights'
Start-Sleep 5
WindowsNow 'lights-cinq-alone'
$listener = Start-Process python -ArgumentList "`"$root\eos\listen.py`"", "`"$out\dmx.jsonl`"", 300, "`"$out\wire.pcap`"" -WindowStyle Hidden -PassThru `
    -RedirectStandardOutput "$out\listen-out.txt" -RedirectStandardError "$out\listen-err.txt"
Mark "listener pid $($listener.Id)"
Start-Sleep 4
Get-NetUDPEndpoint | Where-Object OwningProcess -in $listener.Id, $cinq.Id | Format-Table -AutoSize LocalAddress, LocalPort, OwningProcess | Out-String | Set-Content "$out\ports-lights.txt"
WindowsNow 'lights-start'
Start-Sleep 12
Go 'light 1 preset'
Start-Sleep 14
Shot 'lights-1'
Go 'light 2 fade'
Start-Sleep 14
Shot 'lights-2'
Go 'light 3 blackout'
Start-Sleep 14
Shot 'lights-3'
WindowsNow 'lights-end'
Stop-Process -Id $cinq.Id -Force
Mark 'cinq stopped'
Start-Sleep 12
Mark 'idle after cinq ended'

# Phase 2: network cues against a stand-in console
Mark 'phase network'
function Start-Console([string]$name) {
    $script:console = Start-Process python -ArgumentList "`"$root\eos\fakeeos.py`"", "`"$out\console-$name.txt`"", 300, 3037, 8000 -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput "$out\console-$name-out.txt" -RedirectStandardError "$out\console-$name-err.txt"
    Start-Sleep 2
    Mark "console $name pid $($script:console.Id)"
}
Start-Console 'one'
Start-Cinq 'net.toml' 'net'
$err = "$out\cinq-net-err.txt"
WindowsNow 'net-start'
Start-Sleep 3
Go 'burst, cues 1 to 4'; Start-Sleep 4
Go 'UDP arguments'; Start-Sleep 3
Go 'TCP arguments'; Start-Sleep 4
Go 'single 1'; Start-Sleep 4
Go 'single 2'; Start-Sleep 4
Go 'rapid 1'
Start-Sleep -Milliseconds 400
[System.Windows.Forms.SendKeys]::SendWait(' ')
Mark 'GO rapid 2 (400 ms later)'
Start-Sleep 4
Go 'localhost by name'; Start-Sleep 4
Go 'refused 127.0.0.1:3999'; Warned 'refused' $err; Start-Sleep 3
Shot 'net-refused'
Go 'recovered 1'; Start-Sleep 3
Shot 'net-recovered'
Go 'no answer 203.0.113.7'; Warned 'no answer' $err; Start-Sleep 3
Go 'recovered 2'; Start-Sleep 3
Go 'broadcast'; Warned 'broadcast' $err; Start-Sleep 3
Go 'multicast'; Warned 'multicast' $err; Start-Sleep 3
Go 'any address 0.0.0.0'; Warned 'any address' $err; Start-Sleep 3
Go 'no such name'; Warned 'no such name' $err 8; Start-Sleep 3
Shot 'net-failures'
Go 'recovered 3'; Start-Sleep 4
Go 'before restart'; Start-Sleep 4
Stop-Process -Id $console.Id -Force
Mark 'console one killed'
Start-Console 'two'
Start-Sleep 2
Go 'after restart'; Start-Sleep 4
Go 'UDP to nobody'; Warned 'UDP to nobody' $err; Start-Sleep 3
Go 'last'; Start-Sleep 4
Shot 'net-end'
WindowsNow 'net-end'
Stop-Process -Id $cinq.Id -Force
Stop-Process -Id $console.Id -Force -ErrorAction SilentlyContinue
Mark 'done'

Get-NetFirewallRule | Where-Object { $rulesBefore -notcontains $_.DisplayName } |
    Format-Table -AutoSize DisplayName, Direction, Action, Enabled | Out-String | Set-Content "$out\firewall-new-rules.txt"
Get-NetFirewallRule | Where-Object DisplayName -match 'cinq|python' |
    Format-Table -AutoSize DisplayName, Direction, Action, Enabled | Out-String | Add-Content "$out\firewall-new-rules.txt"
Wait-Process -Id $listener.Id -Timeout 5 -ErrorAction SilentlyContinue
Stop-Process -Id $listener.Id -Force -ErrorAction SilentlyContinue
