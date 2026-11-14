# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Starts cinq with SHOW beside Eos and presses GO for its first cues, GOS
# of them: with show.toml, light cues that listen.py hears on the same
# universe as Eos's, and with network.toml, network cues that fire cues of
# Eos.  cinq.exe comes from cinq-windows\bin in the checkout.
#
# Usage: beside.ps1 [SHOW [GOS]]

param([string]$Show = 'show.toml', [int]$Gos = 2)

. "$PSScriptRoot\ui.ps1"

$exe = "$root\cinq-windows\bin\cinq.exe"

$env:ICED_BACKEND = 'tiny-skia'
Log "starting $exe"
$name = [IO.Path]::GetFileNameWithoutExtension($Show)
$cinq = Start-Process $exe -ArgumentList "`"$root\eos\$Show`"" -PassThru `
    -RedirectStandardOutput "$out\cinq-$name-out.txt" -RedirectStandardError "$out\cinq-$name-err.txt"
$window = $null
for ($i = 0; $i -lt 30 -and -not $window; $i++) {
    Start-Sleep 1
    $window = Get-Process -Id $cinq.Id -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0
}
if (-not $window) { Log 'no cinq window'; Shot 'beside-none'; exit 1 }
Start-Sleep 3
Shot "$name-start"
& "$PSScriptRoot\go.ps1" $name $Gos
