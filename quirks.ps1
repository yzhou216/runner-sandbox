# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a tone of 1 kHz, Left at 0 dB and Right at -6 dB, under the quirks
# of Windows audio devices, and records VB-CABLE through WASAPI's loopback,
# each take as NAME.wav in OUT with a screenshot as NAME.png:
#
# - asio-*: FlexASIO, an ASIO driver that plays on VB-CABLE, whose
#   configuration file changes midway through the cue, to each type of
#   sample and to buffers of 64, 441, 480, and 4096 frames.  FlexASIO
#   reloads the file and asks cinq to reset, as the driver of the Dante
#   Virtual Soundcard does when its settings change.
# - exclusive-*: FlexASIO in WASAPI's exclusive mode, held by another cinq
#   until that one quits, as another program can hold an ASIO driver.
#   WASAPI's loopback hears no exclusive stream, so the screenshots tell.
# - fallback: a show made on another machine, whose outputs name an
#   endpoint that this one lacks, along with the name of VB-CABLE's.
# - stopped: cinq started while the audio service of Windows is stopped,
#   which then starts.
#
# Usage: quirks.ps1 CINQ OUT, with ffmpeg and Python with soundcard on
# PATH, and FlexASIO installed
param([string]$Cinq, [string]$Out)
$ErrorActionPreference = 'Stop'
$Cinq = (Resolve-Path $Cinq).Path
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$work = Join-Path $env:RUNNER_TEMP ([IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force $work | Out-Null
$flexConfig = Join-Path $env:USERPROFILE 'FlexASIO.toml'

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr window, int x, int y, int width, int height, bool repaint);
}
'@
function Shot([string]$name) {
  $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
  $bitmap.Save((Join-Path $Out "$name.png"))
  $graphics.Dispose()
  $bitmap.Dispose()
}
# Five seconds of VB-CABLE through WASAPI's loopback, waited for
function Record([string]$name) {
  Start-Process python -Wait -NoNewWindow `
    -RedirectStandardOutput (Join-Path $Out "record-$name.log") `
    -ArgumentList (Join-Path $PSScriptRoot 'loopback.py'), 'VB-Audio', '5', (Join-Path $Out "$name.wav")
}
function Keys($process, [string]$keys) {
  $process.Refresh()
  [Win]::SetForegroundWindow($process.MainWindowHandle) | Out-Null
  Start-Sleep -Milliseconds 300
  [System.Windows.Forms.SendKeys]::SendWait($keys)
}
# Starts cinq on the show $name, moves its window onto the desktop of the
# runner, which is smaller than it, and fires the cue
function Launch([string]$name) {
  $process = Start-Process $Cinq -PassThru -WorkingDirectory $work -ArgumentList "$name.cinq.toml" `
    -RedirectStandardError (Join-Path $Out "cinq-$name.txt")
  Start-Sleep 10
  $process.Refresh()
  [Win]::MoveWindow($process.MainWindowHandle, 0, 0, 1024, 728, $true) | Out-Null
  Start-Sleep 1
  Keys $process ' '
  Start-Sleep 2
  $process
}
function Flex([string]$sampleType, [int]$buffer, [bool]$exclusive) {
  $lines = @('backend = "Windows WASAPI"')
  if ($buffer -gt 0) { $lines += "bufferSizeSamples = $buffer" }
  $lines += '', '[input]', 'device = ""', '', '[output]', 'channels = 2', "sampleType = `"$sampleType`""
  if ($exclusive) { $lines += 'wasapiExclusiveMode = true' }
  Set-Content -Path $flexConfig -Value $lines -Encoding utf8NoBOM
}
function Show([string]$name, [string]$id, [string]$deviceName) {
  $device = "device = `"$($id -replace '\\', '\\')`"`ndevice_name = `"$deviceName`""
  $show = @"
[[output]]
name = "Left"
$device
channels = "1"

[[output]]
name = "Right"
$device
channels = "2"

[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Tone"
file = "tone.wav"

[list.cue.sends]
Left = 0.0
Right = -6.0
"@
  Set-Content -Path (Join-Path $work "$name.cinq.toml") -Value $show -Encoding utf8NoBOM
  Copy-Item (Join-Path $work "$name.cinq.toml") $Out
}

ffmpeg -loglevel error -y -f lavfi -i 'sine=frequency=1000:sample_rate=48000:duration=300' `
  -af volume=0.5 -ac 2 -c:a pcm_f32le (Join-Path $work 'tone.wav')
Flex 'Int32' 0 $false
& $Cinq --audio-devices > (Join-Path $Out 'devices.tsv')
Get-Content (Join-Path $Out 'devices.tsv')
$cable = Get-Content (Join-Path $Out 'devices.tsv') | Where-Object { $_ -match '^wasapi:.*VB-Audio' } |
  Select-Object -First 1
$cableId, $cableName = $cable -split "`t", 2

# FlexASIO, changed midway through the cue
Show 'asio' 'asio:FlexASIO' 'FlexASIO'
$cinqProcess = Launch 'asio'
Record 'asio-Int32'
foreach ($change in @(
    @{ name = 'asio-Int16'; type = 'Int16'; buffer = 0 },
    @{ name = 'asio-Int24'; type = 'Int24'; buffer = 0 },
    @{ name = 'asio-Float32'; type = 'Float32'; buffer = 0 },
    @{ name = 'asio-buffer-64'; type = 'Int32'; buffer = 64 },
    @{ name = 'asio-buffer-441'; type = 'Int32'; buffer = 441 },
    @{ name = 'asio-buffer-480'; type = 'Int32'; buffer = 480 },
    @{ name = 'asio-buffer-4096'; type = 'Int32'; buffer = 4096 })) {
  Flex $change.type $change.buffer $false
  Start-Sleep 4
  Shot $change.name
  Record $change.name
}
Stop-Process $cinqProcess -ErrorAction SilentlyContinue
Start-Sleep 2

# FlexASIO in exclusive mode, held by another cinq
Flex 'Int16' 0 $true
Show 'holder' 'asio:FlexASIO' 'FlexASIO'
Show 'exclusive' 'asio:FlexASIO' 'FlexASIO'
$holder = Launch 'holder'
$cinqProcess = Launch 'exclusive'
Start-Sleep 4
Shot 'exclusive-held'
Stop-Process $holder -ErrorAction SilentlyContinue
Start-Sleep 6
Shot 'exclusive-freed'
Stop-Process $cinqProcess -ErrorAction SilentlyContinue
Start-Sleep 2
Flex 'Int32' 0 $false

# Show of another machine
Show 'fallback' 'wasapi:{0.0.0.00000000}.{00000000-0000-0000-0000-000000000000}' $cableName
$cinqProcess = Launch 'fallback'
Record 'fallback'
Shot 'fallback'
Stop-Process $cinqProcess -ErrorAction SilentlyContinue
Start-Sleep 2

# Audio service stopped as cinq starts
Show 'stopped' $cableId $cableName
Stop-Service Audiosrv -Force
& $Cinq --audio-devices > (Join-Path $Out 'devices-stopped.tsv') 2> (Join-Path $Out 'devices-stopped.log')
"exit $LASTEXITCODE" | Add-Content (Join-Path $Out 'devices-stopped.log')
$cinqProcess = Launch 'stopped'
Start-Sleep 3
Shot 'stopped-before'
Start-Service Audiosrv
Start-Sleep 6
Record 'stopped'
Shot 'stopped'
Stop-Process $cinqProcess -ErrorAction SilentlyContinue
