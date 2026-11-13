# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Starts listen.py and Eos's offline launcher, chooses Offline w/viz, which
# sends sACN or Art-Net for visualizers without a license key, and waits
# until Eos takes OSC connections on TCP port 3032.  Both programs outlive
# the step, until the runner cleans up at the end of the job.

. "$PSScriptRoot\ui.ps1"

Start-Process python -ArgumentList "`"$root\eos\listen.py`"", "`"$out\dmx.jsonl`"", 3000 -WindowStyle Hidden `
    -RedirectStandardOutput "$out\listen-out.txt" -RedirectStandardError "$out\listen-err.txt"
$launcher = 'C:\Program Files\ETC\EosFamily\v3\ETC_Launch\ETC_LaunchOffline.exe'
Log "starting $launcher"
Start-Process $launcher -WorkingDirectory (Split-Path $launcher)

function Open([int]$port) {
    $client = New-Object System.Net.Sockets.TcpClient
    try { $client.ConnectAsync('127.0.0.1', $port).Wait(1000) -and $client.Connected } catch { $false } finally { $client.Dispose() }
}

$viz = $false
$open = $null
$deadline = (Get-Date).AddMinutes(12)
for ($i = 0; (Get-Date) -lt $deadline; $i++) {
    Start-Sleep 3
    $tree = Snapshot 'start'
    if (-not $viz) { $viz = Press $tree 'w/ ?viz' }
    $null = (Press $tree 'I accept|I agree|accept the') -or (Press $tree '^(OK|Continue|Accept|Start|Launch)$')
    if ($i % 7 -eq 0) {
        Shot ('start-{0:D3}' -f $i)
        Get-Process | Where-Object { $_.ProcessName -match 'eos|etc_|nomad|augment|python' } |
            ForEach-Object { '{0} {1} cpu={2:F0}s ws={3}MB "{4}"' -f $_.ProcessName, $_.Id, $_.CPU, [int]($_.WorkingSet64 / 1MB), $_.MainWindowTitle } |
            Add-Content "$out\processes.txt"
    }
    if (-not $open -and (Open 3032)) { $open = Get-Date; Log 'TCP 3032 accepts connections' }
    if ($open -and ((Get-Date) - $open).TotalSeconds -gt 20) { break }
}
Shot 'start-end'
Get-NetUDPEndpoint | Where-Object LocalPort -in 5568, 6454, 8000, 8001, 3037 | Format-Table -AutoSize | Out-String | Add-Content "$out\ports.txt"
Get-NetTCPConnection -State Listen | Where-Object OwningProcess -in (Get-Process | Where-Object ProcessName -match 'eos').Id |
    Format-Table -AutoSize | Out-String | Add-Content "$out\ports.txt"
if (-not $open) { Log 'Eos never took OSC on TCP 3032'; exit 1 }
