# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Installs the Eos application alone from the parts of ETC's installer in
# eos-parts: the Visual C++ runtimes and the MSI of Eos, with every restart
# suppressed.  The whole installer also brings console patches and USB, dongle,
# and SLP drivers, and the runner that ran it with /S lost its connection.

. "$PSScriptRoot\ui.ps1"

netsh advfirewall set allprofiles state off | Out-Null
Get-NetIPAddress -AddressFamily IPv4 | Format-Table -AutoSize | Out-String | Set-Content "$out\network.txt"
Shot '000-desktop'

$parts = "$root\eos-parts\`$PLUGINSDIR"
function Run([string]$file, [string[]]$arguments) {
    Log "run $file $arguments"
    $p = Start-Process $file -ArgumentList $arguments -PassThru -Wait
    Log "exit $($p.ExitCode)"
}
Run "$parts\VC_redist.x64.exe" '/quiet', '/norestart'
Run "$parts\VC_redist.x86.exe" '/quiet', '/norestart'
Run "$parts\vcredist_2005_sp1_vc8_x86.exe" '/q'
$msi = Get-ChildItem $parts -Filter 'ETC_EosFamily_*.msi' | Select-Object -First 1
Run 'msiexec.exe' '/i', "`"$($msi.FullName)`"", '/qn', '/norestart', 'REBOOT=ReallySuppress', '/l*v', "`"$out\eos-msi.log`""
Shot '001-installed'

Get-ChildItem 'C:\Program Files\ETC' -Recurse -Depth 3 -Directory -ErrorAction SilentlyContinue |
    ForEach-Object FullName | Set-Content "$out\installed.txt"
Get-ChildItem "$env:ProgramData\Microsoft\Windows\Start Menu\Programs", "$env:PUBLIC\Desktop", "$env:USERPROFILE\Desktop" -Recurse -Filter *.lnk -ErrorAction SilentlyContinue |
    ForEach-Object {
        $l = $shell.CreateShortcut($_.FullName)
        "$($_.FullName) -> $($l.TargetPath) $($l.Arguments) in $($l.WorkingDirectory)"
    } | Set-Content "$out\shortcuts.txt"
$(foreach ($key in 'HKLM:\SOFTWARE\ETC', 'HKLM:\SOFTWARE\WOW6432Node\ETC', 'HKCU:\Software\ETC') {
    Get-ChildItem $key -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
        $_.Name
        ($_ | Get-ItemProperty | Select-Object * -ExcludeProperty PS* | Format-List | Out-String).Trim()
    }
}) | Set-Content "$out\registry.txt"
