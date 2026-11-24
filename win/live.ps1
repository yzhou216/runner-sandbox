# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Checks 4 to 7 of the new cinq: file watching with 500 cues, light news,
# a clean exit, and a config.toml that starts with a byte order mark.
# Usage: live.ps1

. "$PSScriptRoot\lib.ps1"

Log "arch $env:PROCESSOR_ARCHITECTURE runner $env:RUNNER_ARCH powershell $($PSVersionTable.PSVersion)"
$go = "[keys]`ngo = [`"Space`", `"F9`"]`n"
$many = New-Dir 'many'
python "$root\win\mkshow.py" $many 500
$lights = New-Dir 'lights'
New-Item -ItemType Directory -Force -Path "$lights\sounds" | Out-Null
Copy-Item "$root\win\shows\lights.cinq.toml" "$lights\show.cinq.toml"
Copy-Item "$root\win\sounds\tone.wav" "$lights\sounds\thunder.wav"
$small = New-Dir 'small'
New-Item -ItemType Directory -Force -Path "$small\sounds" | Out-Null
Copy-Item "$root\win\shows\show.cinq.toml" "$small\show.cinq.toml"
Copy-Item "$root\win\sounds\*.wav" "$small\sounds\"

Use-Cinq 'new'

# 4 File watching in the 500 cue show
$sounds = "$many\sounds"
$p = Start-Cinq '4-watch' $go "$many\show.cinq.toml"
Start-Sleep 20
Shot '4-loaded'
$base = Frame '4-base'

Log 'delete f001.wav'
$sw = [Diagnostics.Stopwatch]::StartNew()
Move-Item "$sounds\f001.wav" "$many\f001.away"
Frames '4a-deleted' $base $sw 8
$gone = Frame '4a-after'
Log 'put f001.wav back'
$sw = [Diagnostics.Stopwatch]::StartNew()
Move-Item "$many\f001.away" "$sounds\f001.wav"
Frames '4b-back' $gone $sw 8
$base = Frame '4b-after'

Log 'replace f002.wav by a longer one, written under a new name and moved over'
Copy-Item "$many\spare\long.wav" "$sounds\f002.new"
$sw = [Diagnostics.Stopwatch]::StartNew()
Move-Item -Force "$sounds\f002.new" "$sounds\f002.wav"
Frames '4c-replaced' $base $sw 8
$base = Frame '4c-after'

Log 'write f003.wav through a handle that shares nothing'
$bytes = [IO.File]::ReadAllBytes("$many\spare\long.wav")
$fs = [IO.File]::Open("$sounds\f003.wav", 'Open', 'ReadWrite', 'None')
$sw = [Diagnostics.Stopwatch]::StartNew()
$fs.SetLength(0); $fs.Write($bytes, 0, $bytes.Length); $fs.Flush()
Frames '4d-held' $base $sw 8
$held = Frame '4d-held-after'
Log 'release f003.wav'
$fs.Dispose()
$sw = [Diagnostics.Stopwatch]::StartNew()
Frames '4e-released' $held $sw 8
Summarize '4-watch'

# 6 Clean exit with the 500 cue show, then with the lights demo
$ms = Close-Cinq $p
Log "EXIT many new: $ms ms"
Summarize '4-watch'
foreach ($which in 'new', 'old') {
    Use-Cinq $which
    foreach ($round in 1..3) {
        $p = Start-Cinq "6-lights-$which-$round" $go "$lights\show.cinq.toml"
        $ms = Close-Cinq $p
        Log "EXIT lights $which round ${round}: $ms ms"
        Summarize "6-lights-$which-$round"
        $p = Start-Cinq "6-many-$which-$round" $go "$many\show.cinq.toml"
        Start-Sleep 20
        $ms = Close-Cinq $p
        Log "EXIT many $which round ${round}: $ms ms"
        Summarize "6-many-$which-$round"
    }
}
Use-Cinq 'new'

# 5 Light news: GO on a light cue, then no input at all
Get-NetAdapter | Format-Table Name, InterfaceDescription, Status | Out-String | Add-Content "$out\adapters.txt"
Get-NetIPAddress -AddressFamily IPv4 | Format-Table InterfaceAlias, IPAddress, PrefixLength | Out-String | Add-Content "$out\adapters.txt"
$p = Start-Cinq '5-lights' $go "$lights\show.cinq.toml"
Shot '5-start'
$base = Frame '5-base'
Key '{F9}' 'F9'
$sw = [Diagnostics.Stopwatch]::StartNew()
Frames '5a-go' $base $sw 8
$base = Frame '5a-after'
Log 'outbound block rule for cinq.exe, then GO again'
New-NetFirewallRule -DisplayName 'sandbox cinq out' -Direction Outbound -Action Block -Program $script:Exe -Profile Any | Out-Null
Key '{F9}' 'F9'
$sw = [Diagnostics.Stopwatch]::StartNew()
Frames '5b-blocked' $base $sw 8
$base = Frame '5b-after'
Remove-NetFirewallRule -DisplayName 'sandbox cinq out'
$virtual = Get-NetAdapter | Where-Object Name -like 'vEthernet*'
Log "virtual adapters: $(($virtual | ForEach-Object Name) -join '; ')"
$virtual | Disable-NetAdapter -Confirm:$false
Start-Sleep 3
Key '{F9}' 'F9'
$sw = [Diagnostics.Stopwatch]::StartNew()
Frames '5c-virtual-down' $base $sw 8
$virtual | Enable-NetAdapter -Confirm:$false
Get-NetIPAddress -AddressFamily IPv4 | Format-Table InterfaceAlias, IPAddress | Out-String | Add-Content "$out\adapters.txt"
Stop-Cinq '5-lights'

# 7 A config.toml that starts with a byte order mark
function Bom([string]$name, [string]$how) {
    $cfg = "$work\cfg-$name"
    $bytes = [IO.File]::ReadAllBytes("$cfg\config.toml")
    $head = ($bytes[0..3] | ForEach-Object { '{0:X2}' -f $_ }) -join ' '
    Log "$name ($how): config.toml starts $head"
}
function Run-Bom([string]$name, [string]$shell, [string]$which) {
    $cfg = "$work\cfg-$name"
    New-Item -ItemType Directory -Force -Path $cfg | Out-Null
    # A file, since a native command line loses the double quotes
    $script = "$work\write-$name.ps1"
    Set-Content -Path $script -Value @(
        "Set-Content -Path '$cfg\config.toml' -Value ('[keys]' + [Environment]::NewLine + 'go = [`"Space`", `"F9`"]') -Encoding utf8"
        '"{0} {1}" -f $PSVersionTable.PSVersion, $PSVersionTable.PSEdition')
    $version = & $shell -NoProfile -ExecutionPolicy Bypass -File $script
    Bom $name "$shell $version"
    Use-Cinq $which
    $p = Start-Cinq $name $null "$small\show.cinq.toml"
    Shot "$name-start"
    Key '{F9}' 'F9'
    Start-Sleep 1
    Shot "$name-after-f9"
    $ms = Close-Cinq $p
    Log "$name closed in $ms ms"
    Summarize $name
}
Run-Bom '7a-ps51-new' 'powershell' 'new'
if (Get-Command pwsh -ErrorAction SilentlyContinue) { Run-Bom '7b-pwsh-new' 'pwsh' 'new' }
Run-Bom '7c-ps51-old' 'powershell' 'old'

# 7d The audio device list, reached by hint mode, with the mark in place
Use-Cinq 'new'
$name = '7d-device'
$cfg = "$work\cfg-$name"
New-Item -ItemType Directory -Force -Path $cfg | Out-Null
Set-Content -Path "$cfg\config.toml" -Value ('[keys]' + [Environment]::NewLine + 'go = ["Space", "F9"]') -Encoding utf8
$p = Start-Cinq $name $null "$small\show.cinq.toml"
[Input]::SetCursorPos(880, 330) | Out-Null
for ($j = 0; $j -lt 14; $j++) { [Place]::mouse_event(0x800, 0, 0, -120, [IntPtr]::Zero); Start-Sleep -Milliseconds 100 }
Start-Sleep 1; Shot "$name-scrolled"
Key 'f' 'f'; Start-Sleep 1; Shot "$name-hints"
Key '{BACKSPACE}' 'Backspace'; Start-Sleep 1
Log 'click the Default device list'
[Input]::Click(862, 403)
Start-Sleep 1; Shot "$name-list"
Key '{DOWN}' 'Down'; Start-Sleep 1; Shot "$name-down"
Key '{ENTER}' 'Enter'; Start-Sleep 1; Shot "$name-chosen"
$ms = Close-Cinq $p
Log "$name closed in $ms ms"
Summarize $name
$bytes = [IO.File]::ReadAllBytes("$cfg\config.toml")
Log "$name config.toml: $(($bytes[0..2] | ForEach-Object { '{0:X2}' -f $_ }) -join ' ') then $([Text.Encoding]::UTF8.GetString($bytes[3..($bytes.Length - 1)]) -replace "`r?`n", ' | ')"
$p = Start-Cinq "$name-again" $null "$small\show.cinq.toml"
Shot "$name-again"
$ms = Close-Cinq $p
Summarize "$name-again"
Log 'done'
