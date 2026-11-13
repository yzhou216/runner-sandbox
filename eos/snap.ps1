# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Takes a screenshot and writes down the windows, under LABEL
#
# Usage: snap.ps1 LABEL

param([string]$Label)

. "$PSScriptRoot\ui.ps1"

Shot $Label
(Windows | ForEach-Object Text) -join "`n" | Set-Content "$out\uia\$Label.txt"
