# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Eight checks of cinq on Windows, one phase each, driven with SendKeys and
# a UDP sender, judged from screenshots and the standard error of cinq.
# Usage: test.ps1

. "$PSScriptRoot\ui.ps1"

$exe = "$root\cinq-windows\bin\cinq.exe"
$env:ICED_BACKEND = 'tiny-skia'
$work = Join-Path $root 'work'
New-Item -ItemType Directory -Force -Path $work | Out-Null

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Place {
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int ht, bool r);
}
'@

$py = (Get-Command python).Source
New-NetFirewallRule -DisplayName 'sandbox python' -Direction Inbound -Action Allow -Program $py -Profile Any | Out-Null
$b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
Log "screen $($b.Width)x$($b.Height) arch $env:PROCESSOR_ARCHITECTURE"

function ReadShared([string]$path) {
    $f = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite,Delete')
    try { (New-Object IO.StreamReader $f).ReadToEnd() } finally { $f.Dispose() }
}

# Starts cinq with a config.toml of CONFIG (none when null) and the show
# SHOW; returns the process
function Start-Cinq([string]$name, $config, [string]$show) {
    CloseWsl
    $cfg = "$work\cfg-$name"
    New-Item -ItemType Directory -Force -Path $cfg | Out-Null
    if ($null -ne $config) { [IO.File]::WriteAllText("$cfg\config.toml", $config) }
    Log "start $name"
    $p = Start-Process $exe -ArgumentList '--config-dir', "`"$cfg`"", "`"$show`"" -PassThru `
        -RedirectStandardOutput "$out\$name-out.txt" -RedirectStandardError "$out\$name-err.txt"
    $w = $null
    for ($i = 0; $i -lt 60 -and -not $w; $i++) {
        Start-Sleep 1
        $w = Get-Process -Id $p.Id -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0
    }
    if (-not $w) { Log "no window for $name"; Shot "$name-nowindow"; return $p }
    Log "window after $i s"
    Start-Sleep 2
    $null = [Place]::MoveWindow($w.MainWindowHandle, 0, 0, 1024, 720, $true)
    Start-Sleep 3
    $p
}

function Key([string]$k, [string]$label) {
    CloseWsl
    $p = Get-Process cinq -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0 | Select-Object -First 1
    if (-not $p) { Log "no cinq window for $label"; return }
    $null = [Input]::SetForegroundWindow($p.MainWindowHandle)
    Start-Sleep -Milliseconds 300
    Log "key $label"
    [System.Windows.Forms.SendKeys]::SendWait($k)
}

function Stop-Cinq([string]$name) {
    Stop-Process -Name cinq -Force -ErrorAction SilentlyContinue
    Start-Sleep 2
    if (Test-Path "$out\$name-err.txt") {
        $e = ReadShared "$out\$name-err.txt"
        Log "stderr of ${name}: $($e.Length) characters"
        Add-Content "$out\stderr-summary.txt" "=== $name ($($e.Length) characters)`n$e"
    }
}

# Copy of a show with its sounds, so that no phase sees another's changes
function New-Show([string]$name, [string]$file = 'show.cinq.toml') {
    $d = "$work\show-$name"
    New-Item -ItemType Directory -Force -Path "$d\sounds" | Out-Null
    Copy-Item "$root\win\shows\$file" "$d\$file"
    Copy-Item "$root\win\sounds\*.wav" "$d\sounds\"
    "$d\$file"
}

# Answers the firewall prompt of windows-11-arm, if it came up
function Prompt([string]$label) {
    $t = [Windows32]::Titles() | Where-Object { $_ -match 'Windows Security|Firewall' }
    Log "prompt check $label : $($t -join '; ')"
    if ($t) { Shot "prompt-$label"; [Input]::Click(414, 527); Start-Sleep 2; Shot "prompt-$label-after" }
}

$go = "[keys]`ngo = [`"Space`", `"F9`"]`n"
Shot 'desktop'

# 1 Silent start, no config.toml
$null = Start-Cinq '1-silent' $null (New-Show '1')
Shot '1-start'
Stop-Cinq '1-silent'

# 2 The retired keymap
$null = Start-Cinq '2-cua' "keymap = `"cua`"`n" (New-Show '2')
Shot '2-start'
Key 'j' 'j'; Start-Sleep 1; Shot '2-after-j'
Key 'o' 'o'; Start-Sleep 1; Shot '2-after-o'
Stop-Cinq '2-cua'

# 3 Hint mode
$null = Start-Cinq '3-hint' $go (New-Show '3')
Shot '3-start'
Key 'f' 'f'; Start-Sleep 1; Shot '3-hints'
$label = (Get-Content "$root\win\hint.txt" -ErrorAction SilentlyContinue | Select-Object -First 1)
if ($label) {
    foreach ($c in $label.ToCharArray()) { Key "$c" "label $c"; Start-Sleep -Milliseconds 400 }
    Start-Sleep 1; Shot '3-label-typed'
    Key 'f' 'f again'; Start-Sleep 1
}
Key 'f' 'f'; Start-Sleep 1; Shot '3-hints-again'
Key '{BACKSPACE}' 'Backspace'; Start-Sleep 1; Shot '3-backspace'
Stop-Cinq '3-hint'

# 4 OSC listener, on the free port first, then with the port taken
$osc = "osc_listen = `"127.0.0.1:53000`"`n$go"
$null = Start-Cinq '4a-osc' $osc (New-Show '4a')
Prompt 'start-4a'
Start-Sleep 3
Prompt 'start-4a-late'
Shot '4a-start'
python "$root\win\osc.py" 127.0.0.1 53000 /go; Log 'sent /go'
Start-Sleep 2; Shot '4a-after-go'
python "$root\win\osc.py" 127.0.0.1 53000 /cue/99/start; Log 'sent /cue/99/start'
Start-Sleep 2; Shot '4a-after-99'
Stop-Cinq '4a-osc'
$hold = Start-Process $py -ArgumentList "`"$root\win\hold.py`"", '127.0.0.1', 53000, 60 -WindowStyle Hidden -PassThru
Start-Sleep 2
$null = Start-Cinq '4b-taken' $osc (New-Show '4b')
Start-Sleep 3
Shot '4b-taken'
Stop-Cinq '4b-taken'
Stop-Process -Id $hold.Id -Force -ErrorAction SilentlyContinue

# 5 File watching
$show5 = New-Show '5'
$tone = Join-Path (Split-Path $show5) 'sounds\tone.wav'
$null = Start-Cinq '5-watch' $go $show5
Shot '5-start'
Move-Item $tone "$tone.away"; Log 'file taken away'
Start-Sleep 4; Shot '5-gone'
Move-Item "$tone.away" $tone; Log 'file put back'
Start-Sleep 4; Shot '5-back'
Stop-Cinq '5-watch'

# 6 Paste
$null = Start-Cinq '6-paste' $go (New-Show '6')
Shot '6-start'
Key 'y' 'y'; Key 'y' 'y'; Key 'p' 'p'
Start-Sleep 1; Shot '6-pasted'
Start-Sleep 5; Shot '6-pasted-later'
Stop-Cinq '6-paste'

# 7 Errors in words: a directory as the show, and a save that cannot write
$dir = "$work\adirectory"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$null = Start-Cinq '7a-directory' $go $dir
Shot '7a-directory'
Stop-Cinq '7a-directory'
$ro = "$work\readonly"
New-Item -ItemType Directory -Force -Path $ro | Out-Null
icacls $ro /deny '*S-1-1-0:(OI)(CI)(WD,AD)' | Out-Null
try { Set-Content "$ro\probe.txt" 'x' -ErrorAction Stop; Log 'probe write worked' } catch { Log "probe write refused: $($_.Exception.Message)" }
$null = Start-Cinq '7b-save' $go "$ro\new.cinq.toml"
Key 'o' 'o'; Start-Sleep 1
Key '{ESC}' 'Escape'; Start-Sleep 1
Key '^s' 'Control+s'
Start-Sleep -Milliseconds 400; Shot '7b-saved-1'
Start-Sleep 1; Shot '7b-saved-2'
Start-Sleep 2; Shot '7b-saved-3'
Stop-Cinq '7b-save'
icacls $ro /remove:d '*S-1-1-0' | Out-Null

# 8 A network cue to a closed port
$null = Start-Cinq '8-net' $go (New-Show '8' 'net.cinq.toml')
Shot '8-start'
Key '{F9}' 'F9'
Start-Sleep 3; Shot '8-after-go'
Start-Sleep 3; Shot '8-after-go-later'
Stop-Cinq '8-net'
Log 'done'
