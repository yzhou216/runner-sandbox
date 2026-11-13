# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Starts cinq with show.toml beside Eos and presses GO for its first two
# light cues, so that listen.py hears cinq's sACN and Art-Net on the same
# universe as Eos's.  cinq.exe comes from cinq-windows\bin in the checkout.

. "$PSScriptRoot\ui.ps1"

$exe = "$root\cinq-windows\bin\cinq.exe"

$env:ICED_BACKEND = 'tiny-skia'
Log "starting $exe"
$cinq = Start-Process $exe -ArgumentList "`"$root\eos\show.toml`"" -PassThru `
    -RedirectStandardOutput "$out\cinq-out.txt" -RedirectStandardError "$out\cinq-err.txt"
$window = $null
for ($i = 0; $i -lt 30 -and -not $window; $i++) {
    Start-Sleep 1
    $window = Get-Process -Id $cinq.Id -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0
}
if (-not $window) { Log 'no cinq window'; Shot 'beside-none'; exit 1 }
Start-Sleep 3
Shot 'beside-start'
foreach ($cue in 1, 2) {
    $null = [Input]::SetForegroundWindow($window.MainWindowHandle)
    Start-Sleep -Milliseconds 300
    Log "GO for cue $cue"
    [System.Windows.Forms.SendKeys]::SendWait(' ')
    Start-Sleep 8
    Shot "beside-cue-$cue"
}
(Windows | ForEach-Object Text) -join "`n" | Set-Content "$out\uia\beside-end.txt"
