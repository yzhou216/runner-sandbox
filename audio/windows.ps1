# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Check 5 and check 3 on Windows, with cinq of the tree in cinq/ cross-built
# for Windows, whose log shows what each key does.  VB-CABLE's playback device
# stands in for a sound card, and loopback.py records it through WASAPI.  Keys
# go through SendKeys to the window, and GO is F5, so that no Space and no
# Escape ever reaches cinq.
#
# - default: Left on channel 1 and Right on channel 2 of the default device,
#   where the outputs view takes `l`, `j`, `o`, `d d`, `p`, and Control+o,
#   the cue plays, and the Levels tab, reached with `g t` twice, takes Tab
#   three times to the send of Right, whose text select-all (Control+A, or Command+A
#   on Darwin) and -12 and Enter set to -12 dB; each
#   take is recorded before and after
# - cable: the same show on the WASAPI device of VB-CABLE by its ID
# - asio-FORMAT: the shows single, two, and boost of clamp.sh on FlexASIO,
#   which plays on VB-CABLE in the format FORMAT of samples (Int16, Int24, or
#   Int32), with clip.py judging the recordings
#
# Usage: windows.ps1 CINQ SOUND OUT, with bash, ffmpeg, and Python with
# soundcard on PATH, SOUND the directory of media.sh's sounds, and OUT the
# directory for the logs, the recordings, and the screenshots
param([string]$Cinq, [string]$Sound, [string]$Out)
$ErrorActionPreference = 'Stop'
$Cinq = (Resolve-Path $Cinq).Path
$Sound = (Resolve-Path $Sound).Path
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$work = Join-Path $env:RUNNER_TEMP ([IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force $work | Out-Null
foreach ($file in 'stereo.wav', 'clip.wav', 'half.wav') { Copy-Item (Join-Path $Sound $file) $work }
# Arguments as they are, braces and all, which Git's bash would glob
$env:MSYS = 'noglob'

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr window, int x, int y, int width, int height, bool repaint);
}
'@
function Mark([string]$phase, [string]$what) {
  "{0}`t{1}" -f ([DateTimeOffset]::Now.ToUnixTimeMilliseconds() / 1000.0), $what |
    Add-Content (Join-Path $Out "$phase.marks")
}
function Shot([string]$name) {
  $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
  $bitmap.Save((Join-Path $Out "$name.png"))
  $graphics.Dispose()
  $bitmap.Dispose()
}
# Loopback of the playback device of VB-CABLE through WASAPI
function Record([string]$name) {
  Start-Process python -PassThru -NoNewWindow `
    -RedirectStandardOutput (Join-Path $Out "$name.rec.log") `
    -ArgumentList (Join-Path $PSScriptRoot 'loopback.py'), 'VB-Audio', '5', (Join-Path $Out "$name.wav")
}
# Types SendKeys text into the window, and marks the phase
function Keys([string]$phase, [string]$label, [string]$keys) {
  $window = (Get-Process cinq).MainWindowHandle
  [Win]::SetForegroundWindow($window) | Out-Null
  Start-Sleep -Milliseconds 300
  Mark $phase $label
  [System.Windows.Forms.SendKeys]::SendWait($keys)
  Start-Sleep -Milliseconds 1500
}
# Starts cinq on the show in the file of the work directory, with a
# configuration directory of its own whose GO key is F5
function Start-Cinq([string]$phase) {
  $cfg = Join-Path $work "cfg-$phase"
  New-Item -ItemType Directory -Force $cfg | Out-Null
  Set-Content -Path (Join-Path $cfg 'config.toml') -Value "keymap = `"vi`"`n`n[keys]`ngo = [`"F5`"]`n" -Encoding utf8NoBOM
  Copy-Item (Join-Path $work "$phase.cinq.toml") $Out
  $process = Start-Process $Cinq -PassThru -WorkingDirectory $work `
    -RedirectStandardError (Join-Path $Out "$phase.log") `
    -ArgumentList '--config-dir', $cfg, "$phase.cinq.toml"
  Mark $phase 'start'
  Start-Sleep 10
  $process.Refresh()
  [Win]::MoveWindow($process.MainWindowHandle, 0, 0, 1024, 728, $true) | Out-Null
  Start-Sleep 1
  $process
}
function Stop-Cinq($process) {
  Stop-Process $process -ErrorAction SilentlyContinue
  Start-Sleep 3
}
function Write-Show([string]$phase, [string[]]$arguments) {
  bash (Join-Path $PSScriptRoot 'shows.sh') @arguments |
    Set-Content -Path (Join-Path $work "$phase.cinq.toml") -Encoding utf8NoBOM
}

& $Cinq --audio-devices > (Join-Path $Out 'devices.tsv')
Get-Content (Join-Path $Out 'devices.tsv')
$cable = Get-Content (Join-Path $Out 'devices.tsv') | Where-Object { $_ -match '^wasapi:.*VB-Audio' } |
  Select-Object -First 1
$cableId, $cableName = $cable -split "`t", 2

# Check 5 on the default device, then on VB-CABLE by its ID
foreach ($phase in 'default', 'cable') {
  if ($phase -eq 'default') { Write-Show $phase @('keys', '-', '-') }
  else { Write-Show $phase @('keys', $cableId, $cableName) }
  $process = Start-Cinq $phase
  Shot "$phase-start"
  Keys $phase 'outputs l' 'l'
  Keys $phase 'outputs j' 'j'
  Keys $phase 'outputs o' 'o'
  Shot "$phase-outputs"
  Keys $phase 'outputs d d' 'dd'
  Keys $phase 'outputs p' 'p'
  Keys $phase 'outputs ctrl-o' '^o'
  Shot "$phase-back"
  Keys $phase 'go F5' '{F5}'
  $recorder = Record "$phase-before"
  Start-Sleep 7
  Shot "$phase-playing"
  Keys $phase 'levels g t 1' 'gt'
  Keys $phase 'levels g t 2' 'gt'
  Shot "$phase-levels"
  1..3 | ForEach-Object { Keys $phase "levels tab $_" '{TAB}' }
  Keys $phase 'levels select all' '^a'
  Keys $phase 'levels send' '-12'
  Keys $phase 'levels enter' '{ENTER}'
  Shot "$phase-sent"
  $recorder = Record "$phase-after"
  Start-Sleep 7
  Stop-Cinq $process
}

# Check 3 on FlexASIO, which plays on VB-CABLE in each format of samples
$asio = Get-Content (Join-Path $Out 'devices.tsv') | Where-Object { $_ -match '^asio:' } | Select-Object -First 1
$asioId, $asioName = $asio -split "`t", 2
foreach ($format in 'Int16', 'Int24', 'Int32') {
  Set-Content -Path (Join-Path $env:USERPROFILE 'FlexASIO.toml') -Encoding utf8NoBOM -Value @"
backend = "Windows WASAPI"

[input]
device = ""

[output]
channels = 2
sampleType = "$format"
"@
  foreach ($show in 'single', 'two', 'boost') {
    $phase = "asio-$format-$show"
    Write-Show $phase @('clamp', $asioId, $asioName, $show)
    $process = Start-Cinq $phase
    Keys $phase 'go F5' '{F5}'
    $recorder = Record $phase
    Start-Sleep 7
    Stop-Cinq $process
  }
}
