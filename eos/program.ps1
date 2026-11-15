# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Starts osc.py with STEPS in the background and waits until it marks the
# show as programmed, while it goes on listening to Eos past this step
#
# Usage: program.ps1 STEPS

param([string]$Steps)

. "$PSScriptRoot\ui.ps1"

# Marks that earlier runs left, which this one has to add to
$marks = @(Select-String -Path "$out\osc.txt" -Pattern '## programmed' -ErrorAction SilentlyContinue).Count
Start-Process python -ArgumentList "`"$root\eos\osc.py`"", "`"$out\osc.txt`"", "`"$root\eos\$Steps`"" -WindowStyle Hidden `
    -RedirectStandardOutput "$out\osc-out-$marks.txt" -RedirectStandardError "$out\osc-err-$marks.txt"
for ($i = 0; $i -lt 90; $i++) {
    Start-Sleep 2
    if (@(Select-String -Path "$out\osc.txt" -Pattern '## programmed' -ErrorAction SilentlyContinue).Count -gt $marks) {
        Log 'programmed'
        Shot 'programmed'
        exit 0
    }
}
Log 'osc.py never programmed Eos'
exit 1
