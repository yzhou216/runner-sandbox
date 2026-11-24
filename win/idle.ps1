# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Checks 2 and 3: processor time of the old and the new cinq at rest, with a
# show of 500 files and with the lights demo, one after the other, twice.
# Usage: idle.ps1

. "$PSScriptRoot\lib.ps1"

$b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
Log "screen $($b.Width)x$($b.Height) arch $env:PROCESSOR_ARCHITECTURE runner $env:RUNNER_ARCH cores $env:NUMBER_OF_PROCESSORS"

$many = New-Dir 'many'
python "$root\win\mkshow.py" $many 500
Log "500 cues: $((Get-ChildItem "$many\sounds").Count) files"
$lights = New-Dir 'lights'
New-Item -ItemType Directory -Force -Path "$lights\sounds" | Out-Null
Copy-Item "$root\win\shows\lights.cinq.toml" "$lights\show.cinq.toml"
Copy-Item "$root\win\sounds\tone.wav" "$lights\sounds\thunder.wav"

# Starts the build WHICH on SHOW, waits 20 s after the window is up, and
# measures the processor time used in the next 60 s with no input
function Measure-Idle([string]$which, [string]$kind, [string]$show, [int]$round) {
    Use-Cinq $which
    $name = "idle-$kind-$which-$round"
    $p = Start-Cinq $name $null $show
    Start-Sleep 20
    Shot "$name-start"
    $before = Cpu $p.Id
    $sw = [Diagnostics.Stopwatch]::StartNew()
    Start-Sleep 60
    $after = Cpu $p.Id
    $wall = $sw.Elapsed.TotalSeconds
    $q = Get-Process -Id $p.Id
    $pct = ($after - $before) / $wall * 100
    $line = '{0},{1},{2},{3},{4:F3},{5:F2},{6:F3},{7},{8}' -f $kind, $which, $round, $env:RUNNER_ARCH, ($after - $before), $wall, $pct, $q.Threads.Count, [int]($q.WorkingSet64 / 1MB)
    Log "IDLE kind=$kind build=$which round=$round cpu=$([math]::Round($after - $before, 3)) s over $([math]::Round($wall, 1)) s = $([math]::Round($pct, 2)) percent of one core, $($q.Threads.Count) threads, $([int]($q.WorkingSet64 / 1MB)) MB"
    Add-Content "$out\idle.csv" $line
    Shot "$name-end"
    $ms = Close-Cinq $p
    Log "closed in $ms ms"
    Summarize $name
}

Add-Content "$out\idle.csv" 'kind,build,round,arch,cpu_s,wall_s,percent,threads,mb'
foreach ($round in 1, 2) {
    foreach ($which in 'old', 'new') { Measure-Idle $which 'files' "$many\show.cinq.toml" $round }
    foreach ($which in 'old', 'new') { Measure-Idle $which 'lights' "$lights\show.cinq.toml" $round }
}
Log 'done'
