# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays the show of show.sh on VB-CABLE, which Over on channel 3 overruns,
# takes screenshots of the main window, the Levels tab, the outputs view,
# and the inspector of Stage L, and records VB-CABLE through WASAPI's
# loopback as house.wav, all in OUT.
#
# Usage: windows.ps1 CINQ OUT, with bash, ffmpeg, and Python with
# soundcard on PATH
param([string]$Cinq, [string]$Out)
$ErrorActionPreference = 'Stop'
$Cinq = (Resolve-Path $Cinq).Path
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$work = Join-Path $env:RUNNER_TEMP ([IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force $work | Out-Null

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr window, int x, int y, int width, int height, bool repaint);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
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
function Keys($process, [string]$keys) {
  $process.Refresh()
  [Win]::SetForegroundWindow($process.MainWindowHandle) | Out-Null
  Start-Sleep -Milliseconds 300
  [System.Windows.Forms.SendKeys]::SendWait($keys)
  Start-Sleep 1
}

ffmpeg -loglevel error -y -f lavfi -i 'sine=frequency=1000:sample_rate=48000:duration=60' `
  -af volume=0.5 -ac 2 -c:a pcm_f32le (Join-Path $work 'tone.wav')
& $Cinq --audio-devices > (Join-Path $Out 'devices.tsv')
$cable = Get-Content (Join-Path $Out 'devices.tsv') | Where-Object { $_ -match '^wasapi:.*VB-Audio' } |
  Select-Object -First 1
$id, $name = $cable -split "`t", 2
$missing = 'wasapi:{0.0.0.00000000}.{11111111-2222-3333-4444-555555555555}'
# Arguments as they are, braces and all, which Git's bash would glob
$env:MSYS = 'noglob'
bash (Join-Path $PSScriptRoot 'show.sh') $id $name 3 $missing |
  Set-Content -Path (Join-Path $work 'ui.cinq.toml') -Encoding utf8NoBOM
Copy-Item (Join-Path $work 'ui.cinq.toml') $Out

$process = Start-Process $Cinq -PassThru -WorkingDirectory $work -ArgumentList 'ui.cinq.toml'
Start-Sleep 10
$process.Refresh()
[Win]::MoveWindow($process.MainWindowHandle, 0, 0, 1024, 728, $true) | Out-Null
Start-Sleep 1
Keys $process ' '
$recorder = Start-Process python -PassThru -NoNewWindow `
  -RedirectStandardOutput (Join-Path $Out 'record.log') `
  -ArgumentList (Join-Path $PSScriptRoot 'loopback.py'), 'VB-Audio', '5', (Join-Path $Out 'house.wav')
Start-Sleep 1
Shot 'main'
Keys $process 'gt'
Keys $process 'gt'
Shot 'levels'
Keys $process 'l'
Shot 'outputs'
Keys $process 'jjj'
Shot 'inspector'
# Device list of House L, which the list of devices taken as cinq started
# should hold, before, while, and after it opens on a click, about 186
# pixels above the bottom of the window
Keys $process 'kkk'
Shot 'house'
[Win]::SetCursorPos(440, 542) | Out-Null
Start-Sleep -Milliseconds 100
[Win]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
[Win]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep 2
Shot 'device-list'
[Win]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
[Win]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep 2
Shot 'house-listed'
$recorder.WaitForExit()
Stop-Process $process -ErrorAction SilentlyContinue

# Show made on another machine, whose House L and House R name an endpoint
# that this one lacks, along with the name of VB-CABLE's, which the Device
# list of House L should choose by that name
$foreign = 'wasapi:{0.0.0.00000000}.{99999999-8888-7777-6666-555555555555}'
bash (Join-Path $PSScriptRoot 'show.sh') $foreign $name 3 $missing |
  Set-Content -Path (Join-Path $work 'foreign.cinq.toml') -Encoding utf8NoBOM
Copy-Item (Join-Path $work 'foreign.cinq.toml') $Out
$process = Start-Process $Cinq -PassThru -WorkingDirectory $work -ArgumentList 'foreign.cinq.toml'
Start-Sleep 10
$process.Refresh()
[Win]::MoveWindow($process.MainWindowHandle, 0, 0, 1024, 728, $true) | Out-Null
Start-Sleep 1
Keys $process 'l'
Shot 'foreign'
Stop-Process $process -ErrorAction SilentlyContinue
