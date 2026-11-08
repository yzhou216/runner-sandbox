# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Control on Windows: the programs with the shim, which there finds every
# function in the system, and a screenshot of the counter drawn by
# tiny-skia, to tell ReactOS's faults apart from iced's own.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
New-Item -ItemType Directory -Force out | Out-Null

function Save-Screen($path) {
    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save($path)
}

foreach ($arch in 'i686', 'x86_64') {
    $programs = "programs-$arch"
    foreach ($run in @('HELLO_SH'), @('ONE_SH'), @('THRDS_SH', 'sleep')) {
        $log = "out\$($run -join '-')-$arch.txt"
        $arguments = @($run | Select-Object -Skip 1)
        & "$programs\$($run[0]).EXE" @arguments *> $log
        "exit $LASTEXITCODE" | Add-Content $log
        Get-Content $log | ForEach-Object { "$arch $($run[0]): $_" }
    }
    $env:ICED_BACKEND = 'tiny-skia'
    $counter = Start-Process "$programs\COUNT_SH.EXE" -NoNewWindow -PassThru `
        -RedirectStandardError "out\counter-$arch.err.txt"
    Start-Sleep 15
    Save-Screen "out\counter-$arch.png"
    Stop-Process $counter
    Remove-Item Env:ICED_BACKEND
}
