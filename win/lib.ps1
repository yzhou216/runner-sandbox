# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Helpers of idle.ps1 and live.ps1: starting and stopping a cinq build, keys,
# processor time, and screenshots whose regions are hashed to time a change.
# Dot-source it.

. "$PSScriptRoot\ui.ps1"

$env:ICED_BACKEND = 'tiny-skia'
$work = Join-Path $root 'work'
New-Item -ItemType Directory -Force -Path $work | Out-Null

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Place {
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, int d, IntPtr e);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int ht, bool r);
}
'@

if ($env:RUNNER_ARCH -eq 'ARM64') {
    Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$PSScriptRoot\closer.ps1`"", 3300 -WindowStyle Hidden
}

# Picks the build, 'new' or 'old', that Start-Cinq runs
function Use-Cinq([string]$which) {
    $script:Exe = "$root\cinq-$which\bin\cinq.exe"
    $script:Which = $which
}

function ReadShared([string]$path) {
    $f = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite,Delete')
    try { (New-Object IO.StreamReader $f).ReadToEnd() } finally { $f.Dispose() }
}

# Starts the picked build with a config.toml of CONFIG (none when null) in
# a config directory of its own and the show SHOW; returns the process
function Start-Cinq([string]$name, $config, [string]$show) {
    CloseWsl
    $cfg = "$work\cfg-$name"
    New-Item -ItemType Directory -Force -Path $cfg | Out-Null
    if ($null -ne $config) { [IO.File]::WriteAllText("$cfg\config.toml", $config) }
    Log "start $name ($script:Which)"
    $p = Start-Process $script:Exe -ArgumentList '--config-dir', "`"$cfg`"", "`"$show`"" -PassThru `
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
    Start-Sleep 5
    $p
}

function Key([string]$k, [string]$label) {
    CloseWsl
    Start-Sleep -Milliseconds 500
    $p = Get-Process cinq -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0 | Select-Object -First 1
    if (-not $p) { Log "no cinq window for $label"; return }
    $null = [Input]::SetForegroundWindow($p.MainWindowHandle)
    Start-Sleep -Milliseconds 300
    Log "key $label"
    [System.Windows.Forms.SendKeys]::SendWait($k)
}

# Ends cinq and files its standard error in the summary
function Stop-Cinq([string]$name) {
    Stop-Process -Name cinq -Force -ErrorAction SilentlyContinue
    Start-Sleep 2
    Summarize $name
}

function Summarize([string]$name) {
    if (Test-Path "$out\$name-err.txt") {
        $e = ReadShared "$out\$name-err.txt"
        Log "stderr of ${name}: $($e.Length) characters"
        Add-Content "$out\stderr-summary.txt" "=== $name ($($e.Length) characters)`n$e"
    }
}

# Asks the main window to close, as the close button does (WM_CLOSE), and
# returns the milliseconds until the process has ended, or -1 if it has not
# ended after LIMIT seconds, in which case it is killed
function Close-Cinq($p, [int]$limit = 15) {
    $null = $p.CloseMainWindow()
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while (-not $p.HasExited -and $sw.Elapsed.TotalSeconds -lt $limit) { Start-Sleep -Milliseconds 20 }
    if ($p.HasExited) { return [int]$sw.Elapsed.TotalMilliseconds }
    $threads = (Get-Process -Id $p.Id -ErrorAction SilentlyContinue).Threads.Count
    Log "still running after ${limit} s, $threads threads"
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    -1
}

# Processor time of cinq in seconds, summed over the processes named cinq
function Cpu([int]$id) {
    (Get-Process -Id $id).TotalProcessorTime.TotalSeconds
}

# Copy of a directory of sounds named SOUNDS, for a show whose file is FILE
function New-Dir([string]$name) {
    $d = "$work\$name"
    New-Item -ItemType Directory -Force -Path $d | Out-Null
    $d
}

# Takes a screenshot as Shot does and returns SHA-1 hashes of two regions of
# it: the first three rows of the cue table, and the status bar.  They
# change when the state of a cue, its action, or the warning count does.
$script:sha = [Security.Cryptography.SHA1]::Create()
function Frame([string]$name) {
    $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size)
    $bmp.Save("$out\shots\$name.png", [System.Drawing.Imaging.ImageFormat]::Png)
    $hash = foreach ($r in @((New-Object System.Drawing.Rectangle 0, 205, 760, 125), (New-Object System.Drawing.Rectangle 0, 675, 1024, 40))) {
        $ms = New-Object IO.MemoryStream
        $bmp.Clone($r, $bmp.PixelFormat).Save($ms, [System.Drawing.Imaging.ImageFormat]::Bmp)
        [BitConverter]::ToString($script:sha.ComputeHash($ms.ToArray()))
        $ms.Dispose()
    }
    $g.Dispose(); $bmp.Dispose()
    [pscustomobject]@{ Table = $hash[0]; Bar = $hash[1] }
}

# Takes a frame about every 400 ms for SECONDS seconds, named TAG and the
# milliseconds since the stopwatch SW started, and logs when the table and
# the bar first differ from BASE
function Frames([string]$tag, $base, $sw, [int]$seconds) {
    $end = $sw.Elapsed.TotalSeconds + $seconds
    $table = $null; $bar = $null
    while ($sw.Elapsed.TotalSeconds -lt $end) {
        $ms = [int]$sw.Elapsed.TotalMilliseconds
        $f = Frame ('{0}-{1:D5}' -f $tag, $ms)
        if (-not $table -and $f.Table -ne $base.Table) { $table = $ms; Log "$tag table changed at $ms ms" }
        if (-not $bar -and $f.Bar -ne $base.Bar) { $bar = $ms; Log "$tag bar changed at $ms ms" }
        Start-Sleep -Milliseconds 400
    }
    if (-not $table) { Log "$tag table did not change in $seconds s" }
    if (-not $bar) { Log "$tag bar did not change in $seconds s" }
}
