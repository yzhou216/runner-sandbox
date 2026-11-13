# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a show on CABLE Input, the playback device of VB-CABLE, a virtual
# WASAPI device, as the Dante Virtual Soundcard in WDM mode offers each
# pair of its channels as a device of its own, and records CABLE Output,
# which loops back what CABLE Input plays.  Midway through the cue, the
# device is disabled and enabled again, as the Dante Virtual Soundcard
# goes away and comes back when it restarts.  cinq should take the device
# up again by itself and go on with the cue from where it is.
#
# Usage: wasapi.ps1 CINQ SOUND OUT, with ffmpeg on PATH, CINQ the program,
# SOUND the sound of the cue, and OUT the directory for the recordings,
# the screenshots, and marks.tsv
param([string]$Cinq, [string]$Sound, [string]$Out)
$ErrorActionPreference = 'Stop'
$Cinq = (Resolve-Path $Cinq).Path
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$work = Join-Path $env:RUNNER_TEMP 'wasapi'
New-Item -ItemType Directory -Force $work | Out-Null
Copy-Item $Sound (Join-Path $work 'music.flac')

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
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
# Recording device of VB-CABLE as DirectShow names it, which loops back
# what its playback device plays
$listed = (& ffmpeg -hide_banner -list_devices true -f dshow -i dummy 2>&1) | Out-String
Set-Content (Join-Path $Out 'dshow.txt') $listed
$capture = [regex]::Matches($listed, '"([^"]*VB-Audio[^"]*)" \(audio\)') |
  ForEach-Object { $_.Groups[1].Value } | Select-Object -First 1
"capture device: $capture"
function Record([string]$name) {
  Start-Process ffmpeg -PassThru -NoNewWindow `
    -RedirectStandardError (Join-Path $Out "$name.log") `
    -ArgumentList '-y', '-f', 'dshow', '-i', "audio=`"$capture`"",
      '-t', '10', (Join-Path $Out "$name.wav")
}
function Keys([string]$keys) {
  $window = (Get-Process cinq).MainWindowHandle
  [Win]::SetForegroundWindow($window) | Out-Null
  Start-Sleep -Milliseconds 300
  [System.Windows.Forms.SendKeys]::SendWait($keys)
}

& $Cinq --audio-devices > (Join-Path $Out 'devices.tsv')
Get-Content (Join-Path $Out 'devices.tsv')
$line = Get-Content (Join-Path $Out 'devices.tsv') | Where-Object { $_ -match "`t.*VB-Audio" } |
  Select-Object -First 1
if (-not $line) { throw 'cinq lists no device of VB-CABLE' }
$id, $name = $line -split "`t", 2

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

$device = Get-PnpDevice -FriendlyName '*VB-Audio Virtual Cable*' | Select-Object -First 1
"device: $($device.InstanceId) $($device.FriendlyName) $($device.Status)" | Tee-Object (Join-Path $Out 'pnp.txt')

$cinqProcess = Start-Process $Cinq -PassThru -WorkingDirectory $work -ArgumentList 'cable.cinq.toml' `
  -RedirectStandardError (Join-Path $Out 'cinq-err.txt')
Start-Sleep 10
Keys ' '
Mark 'go'
$recorder = Record 'before'
Start-Sleep 6
Shot 'playing'
Stop-Process $recorder -ErrorAction SilentlyContinue
Mark 'gone'
Disable-PnpDevice -InstanceId $device.InstanceId -Confirm:$false
Start-Sleep 6
Shot 'gone'
Enable-PnpDevice -InstanceId $device.InstanceId -Confirm:$false
Mark 'back'
# cinq tries every two seconds
Start-Sleep 5
$recorder = Record 'after'
Start-Sleep 6
Shot 'back'
Stop-Process $recorder -ErrorAction SilentlyContinue
Mark 'end'
Stop-Process $cinqProcess -ErrorAction SilentlyContinue
