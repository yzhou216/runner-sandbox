# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a show on a device that cinq lists, whose line in the list MATCH
# matches: the playback device of VB-CABLE, a virtual WASAPI device, as the
# Dante Virtual Soundcard in WDM mode offers each pair of its channels as a
# device of its own, or FlexASIO, an ASIO driver that plays on it, as the
# Dante Virtual Soundcard offers all of its channels as an ASIO driver.
# What VB-CABLE plays is recorded through WASAPI's loopback.  With
# -Revive, VB-CABLE is taken away midway through the cue and installed
# again, as the Dante Virtual Soundcard goes away and comes back when it
# restarts, and cinq should take the device up again by itself and go on
# with the cue from where it is.
#
# Usage: wasapi.ps1 CINQ SOUND OUT MATCH [-Revive], with ffmpeg and Python
# with soundcard on PATH, CINQ the program, SOUND the sound of the cue, and
# OUT the directory for the recordings, the screenshots, and marks.tsv
param([string]$Cinq, [string]$Sound, [string]$Out, [string]$Match, [switch]$Revive)
$ErrorActionPreference = 'Stop'
$Cinq = (Resolve-Path $Cinq).Path
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$work = Join-Path $env:RUNNER_TEMP ([IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force $work | Out-Null
Copy-Item $Sound (Join-Path $work 'music.flac')

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr window, int x, int y, int width, int height, bool repaint);
}
'@
function Mark([string]$what) {
  "{0}`t{1}" -f [DateTimeOffset]::Now.ToUnixTimeMilliseconds(), $what |
    Add-Content (Join-Path $Out 'marks.tsv')
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
# Loopback of the playback device of VB-CABLE through WASAPI, since
# DirectShow cannot open its recording device on the runner
function Record([string]$name) {
  Start-Process python -PassThru -NoNewWindow `
    -RedirectStandardOutput (Join-Path $Out "$name.log") `
    -ArgumentList (Join-Path $PSScriptRoot 'loopback.py'), 'VB-Audio', '5', (Join-Path $Out "$name.wav")
}
function Keys([string]$keys) {
  $window = (Get-Process cinq).MainWindowHandle
  [Win]::SetForegroundWindow($window) | Out-Null
  Start-Sleep -Milliseconds 300
  [System.Windows.Forms.SendKeys]::SendWait($keys)
}

& $Cinq --audio-devices > (Join-Path $Out 'devices.tsv')
Get-Content (Join-Path $Out 'devices.tsv')
$line = Get-Content (Join-Path $Out 'devices.tsv') | Where-Object { $_ -match $Match } |
  Select-Object -First 1
if (-not $line) { throw "cinq lists no device that matches $Match" }
$id, $name = $line -split "`t", 2
"playing on $id, $name"

$show = @"
[[output]]
name = "Left"
device = "$($id -replace '\\', '\\')"
device_name = "$name"
channels = "1"

[[output]]
name = "Right"
device = "$($id -replace '\\', '\\')"
device_name = "$name"
channels = "2"

[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Music"
file = "music.flac"

[list.cue.sends]
Left = 0.0
Right = -6.0
"@
Set-Content -Path (Join-Path $work 'cable.cinq.toml') -Value $show -Encoding utf8NoBOM
Copy-Item (Join-Path $work 'cable.cinq.toml') $Out

$cinqProcess = Start-Process $Cinq -PassThru -WorkingDirectory $work -ArgumentList 'cable.cinq.toml' `
  -RedirectStandardError (Join-Path $Out 'cinq-err.txt')
Start-Sleep 10
# Whole window on the desktop of the runner, which is smaller than it
[Win]::MoveWindow((Get-Process cinq).MainWindowHandle, 0, 0, 1024, 728, $true) | Out-Null
Start-Sleep 1
Keys ' '
Mark 'go'
$recorder = Record 'before'
Start-Sleep 6
Shot 'playing'
Stop-Process $recorder -ErrorAction SilentlyContinue
if ($Revive) {
  # devcon and the driver that LABSN/sound-ci-helpers left among the actions
  $helpers = Get-ChildItem -Recurse -Path (Join-Path $env:RUNNER_WORKSPACE '..\_actions') -Filter devcon.exe |
    Select-Object -First 1
  $devcon = $helpers.FullName
  $inf = Get-ChildItem -Recurse -Path $helpers.DirectoryName -Filter vbMmeCable64_win7.inf |
    Select-Object -First 1
  Mark 'gone'
  & $devcon remove 'ROOT\MEDIA\*' | Tee-Object -Append (Join-Path $Out 'pnp.txt')
  Start-Sleep 6
  & $Cinq --audio-devices > (Join-Path $Out 'devices-gone.tsv')
  Shot 'gone'
  & $devcon install $inf.FullName VBAudioVACWDM | Tee-Object -Append (Join-Path $Out 'pnp.txt')
  Mark 'back'
  # cinq tries every two seconds
  Start-Sleep 6
  & $Cinq --audio-devices > (Join-Path $Out 'devices-back.tsv')
  $recorder = Record 'after'
  Start-Sleep 6
  Shot 'back'
  Stop-Process $recorder -ErrorAction SilentlyContinue
}
Mark 'end'
Stop-Process $cinqProcess -ErrorAction SilentlyContinue
