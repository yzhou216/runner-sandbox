# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a tone of 1 kHz, Left at 0 dB and Right at -6 dB, on VB-CABLE
# through WASAPI with each cinq.exe in EXES, for ten seconds, while eight
# loops keep the processors busy, as a show machine that decodes video
# meanwhile.  WASAPI's loopback records VB-CABLE at high priority, so that
# the recorder itself keeps up, as NAME.wav in OUT, and the priorities of
# the threads of cinq go to NAME-threads.txt.
#
# Usage: stress.ps1 OUT NAME=EXE..., with ffmpeg and Python with soundcard
# on PATH
param([string]$Out, [Parameter(ValueFromRemainingArguments)][string[]]$Exes)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$work = Join-Path $env:RUNNER_TEMP ([IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force $work | Out-Null

Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
}
'@
ffmpeg -loglevel error -y -f lavfi -i 'sine=frequency=1000:sample_rate=48000:duration=120' `
  -af volume=0.5 -ac 2 -c:a pcm_f32le (Join-Path $work 'tone.wav')

foreach ($build in $Exes) {
  $name, $exe = $build -split '=', 2
  $exe = (Resolve-Path $exe).Path
  & $exe --audio-devices > (Join-Path $Out "devices-$name.tsv")
  $cable = Get-Content (Join-Path $Out "devices-$name.tsv") | Where-Object { $_ -match '^wasapi:.*VB-Audio' } |
    Select-Object -First 1
  $id, $deviceName = $cable -split "`t", 2
  $device = "device = `"$id`"`ndevice_name = `"$deviceName`""
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
  Set-Content -Path (Join-Path $work 'stress.cinq.toml') -Value $show -Encoding utf8NoBOM
  $cinq = Start-Process $exe -PassThru -WorkingDirectory $work -ArgumentList 'stress.cinq.toml'
  Start-Sleep 10
  $cinq.Refresh()
  [Win]::SetForegroundWindow($cinq.MainWindowHandle) | Out-Null
  Start-Sleep -Milliseconds 300
  [System.Windows.Forms.SendKeys]::SendWait(' ')
  Start-Sleep 2
  $busy = 1..8 | ForEach-Object {
    Start-Process pwsh -PassThru -WindowStyle Hidden -ArgumentList '-NoProfile', '-Command', 'while ($true) {}'
  }
  $recorder = Start-Process python -PassThru -NoNewWindow `
    -RedirectStandardOutput (Join-Path $Out "record-$name.log") `
    -ArgumentList (Join-Path $PSScriptRoot 'loopback.py'), 'VB-Audio', '10', (Join-Path $Out "$name.wav")
  $recorder.PriorityClass = 'High'
  Start-Sleep 3
  $cinq.Refresh()
  $cinq.Threads | Select-Object Id, BasePriority, CurrentPriority, PriorityLevel |
    Format-Table -AutoSize | Out-String | Set-Content (Join-Path $Out "$name-threads.txt")
  $recorder.WaitForExit()
  $busy | Stop-Process -ErrorAction SilentlyContinue
  Stop-Process $cinq -ErrorAction SilentlyContinue
  Start-Sleep 3
}
