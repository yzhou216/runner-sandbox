# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Clears what covers the desktop of windows-11-arm at first logon: the
# privacy window of WWAHost and then the Start menu that it leaves open
#
# Usage: prep.ps1

. "$PSScriptRoot\ui.ps1"

Shot 'prep-0-first'
Get-Process WWAHost -ErrorAction SilentlyContinue | ForEach-Object { Log "WWAHost $($_.Id)" }
Stop-Process -Name WWAHost -Force -ErrorAction SilentlyContinue
Start-Sleep 2
[System.Windows.Forms.SendKeys]::SendWait('{ESC}')
Start-Sleep 1
CloseWsl
Shot 'prep-1-cleared'
