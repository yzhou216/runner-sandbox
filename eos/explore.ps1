# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Installs Eos, starts it, and drives it over OSC while recording what it
# sends as sACN or Art-Net.  Every dialog on the way is written down from
# UI Automation and screenshots, and a few answers are given by name.
#
# Usage: explore.ps1 STEPS

param([string]$Steps)

$ErrorActionPreference = 'Continue'
$root = (Resolve-Path "$PSScriptRoot\..").Path
$out = Join-Path $root 'out'
New-Item -ItemType Directory -Force -Path $out, "$out\shots", "$out\uia" | Out-Null
$start = Get-Date

function Log([string]$m) {
    $line = '{0,7:F1} {1}' -f ((Get-Date) - $start).TotalSeconds, $m
    Write-Host $line
    Add-Content "$out\log.txt" $line
}

Add-Type -AssemblyName System.Drawing, System.Windows.Forms, UIAutomationClient, UIAutomationTypes
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Input {
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, IntPtr e);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    public static void Click(int x, int y) {
        SetCursorPos(x, y);
        mouse_event(2, 0, 0, 0, IntPtr.Zero);
        mouse_event(4, 0, 0, 0, IntPtr.Zero);
    }
}
'@

function Shot([string]$name) {
    try {
        $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
        $bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size)
        $bmp.Save("$out\shots\$name.png", [System.Drawing.Imaging.ImageFormat]::Png)
        $g.Dispose(); $bmp.Dispose()
    } catch { Log "shot failed: $_" }
}

$walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
$scope = [System.Windows.Automation.TreeScope]::Children
$all = [System.Windows.Automation.Condition]::TrueCondition

# Each element as text, with the element kept for pressing it later
function Walk($element, [int]$depth, $found) {
    $child = $walker.GetFirstChild($element)
    while ($null -ne $child -and $found.Count -lt 400) {
        try {
            $d = $child.Current
            $kind = $d.ControlType.ProgrammaticName -replace 'ControlType\.', ''
            $found.Add([pscustomobject]@{
                Element = $child; Kind = $kind; Name = $d.Name
                Text = ('  ' * $depth) + "$kind '$($d.Name)' $($d.BoundingRectangle)"
            })
            if ($depth -lt 9) { Walk $child ($depth + 1) $found }
            $child = $walker.GetNextSibling($child)
        } catch { $child = $null }
    }
}

function Windows {
    $found = New-Object System.Collections.Generic.List[object]
    foreach ($w in [System.Windows.Automation.AutomationElement]::RootElement.FindAll($scope, $all)) {
        try {
            $c = $w.Current
            $p = try { (Get-Process -Id $c.ProcessId -ErrorAction Stop).ProcessName } catch { '?' }
            $found.Add([pscustomobject]@{
                Element = $w; Kind = 'Window'; Name = $c.Name; Process = $p
                Text = "WINDOW [$p] '$($c.Name)' $($c.ClassName) $($c.BoundingRectangle)"
            })
            if ($p -notmatch '^(explorer|ShellExperienceHost|SearchHost|StartMenuExperienceHost|TextInputHost|powershell|pwsh|bash|cmd|conhost|Runner\.Worker|Runner\.Listener)$') {
                Walk $w 1 $found
            }
        } catch {}
    }
    $found
}

$script:lastTree = ''
$script:treeCount = 0
function Snapshot([string]$label) {
    $tree = Windows
    $text = ($tree | ForEach-Object Text) -join "`n"
    if ($text -ne $script:lastTree) {
        $script:lastTree = $text
        $script:treeCount++
        $name = '{0:D3}-{1}' -f $script:treeCount, $label
        Set-Content "$out\uia\$name.txt" $text
        Shot $name
        Log "tree $name changed"
    }
    $tree
}

function Press($tree, [string]$pattern, [string]$kinds = 'Button|RadioButton|CheckBox|ListItem|Hyperlink|MenuItem|TabItem') {
    $hit = $tree | Where-Object { $_.Kind -match $kinds -and $_.Name -match $pattern } | Select-Object -First 1
    if (-not $hit) { return $false }
    $e = $hit.Element
    Log "press $($hit.Kind) '$($hit.Name)'"
    try {
        $e.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke(); return $true
    } catch {}
    try {
        $e.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle(); return $true
    } catch {}
    try {
        $e.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select(); return $true
    } catch {}
    $r = $e.Current.BoundingRectangle
    [Input]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
    $true
}

netsh advfirewall set allprofiles state off | Out-Null
Get-NetIPAddress -AddressFamily IPv4 | Format-Table -AutoSize | Out-String | Add-Content "$out\network.txt"
Shot '000-desktop'

# Install

$installer = Get-ChildItem "$root\eos-installer" -Filter *.exe | Select-Object -First 1
Log "installing $($installer.Name) with /S"
$setup = Start-Process $installer.FullName -ArgumentList '/S' -PassThru
$deadline = (Get-Date).AddMinutes(30)
while ((Get-Date) -lt $deadline) {
    Start-Sleep 10
    $tree = Snapshot 'install'
    $restart = $tree | Where-Object { $_.Name -match 'restart|reboot' }
    if ($restart) {
        $null = (Press $tree '^(No|Later|Restart Later|Close|Cancel)$')
    } else {
        $null = (Press $tree 'I accept|I agree|accept the') -or (Press $tree '^(&?Install|&?Next.*|OK|Continue|&?Close|&?Finish)$')
    }
    $busy = Get-Process | Where-Object { $_.ProcessName -match 'msiexec|ETC_|vc_redist|CertMgr' }
    if ($setup.HasExited -and -not ($busy | Where-Object ProcessName -ne 'msiexec')) {
        Log "installer exited with $($setup.ExitCode)"
        break
    }
}
Get-ChildItem $env:TEMP -Recurse -Filter 'EosInstall*' -ErrorAction SilentlyContinue | Copy-Item -Destination $out
Get-ChildItem 'C:\Program Files\ETC', 'C:\Program Files (x86)\ETC' -Recurse -Depth 2 -ErrorAction SilentlyContinue |
    ForEach-Object FullName | Set-Content "$out\installed.txt"
$shell = New-Object -ComObject WScript.Shell
$links = Get-ChildItem "$env:ProgramData\Microsoft\Windows\Start Menu\Programs", "$env:PUBLIC\Desktop", "$env:USERPROFILE\Desktop" -Recurse -Filter *.lnk -ErrorAction SilentlyContinue
$links | ForEach-Object {
    $l = $shell.CreateShortcut($_.FullName)
    "$($_.FullName) -> $($l.TargetPath) $($l.Arguments) in $($l.WorkingDirectory)"
} | Set-Content "$out\shortcuts.txt"

# Start

$eos = $links | Where-Object { $_.BaseName -match 'Eos|nomad' -and $_.BaseName -notmatch 'Uninstall|Augment|Help|Manual|Release|Widget' } | Select-Object -First 1
if (-not $eos) { Log 'no shortcut for Eos'; exit 1 }
$link = $shell.CreateShortcut($eos.FullName)
pktmon filter remove | Out-Null
foreach ($port in 5568, 6454, 3032, 8000, 8001) { pktmon filter add "p$port" -p $port | Out-Null }
pktmon start --capture --pkt-size 0 --file-name "$out\eos.etl" | Out-Null
$listen = Start-Process python -ArgumentList "`"$root\eos\listen.py`"", "`"$out\dmx.jsonl`"", 2400 -PassThru -NoNewWindow -RedirectStandardError "$out\listen-err.txt"
Log "starting $($eos.Name): $($link.TargetPath) $($link.Arguments)"
if ($link.Arguments) {
    Start-Process $link.TargetPath -ArgumentList $link.Arguments -WorkingDirectory $link.WorkingDirectory
} else {
    Start-Process $link.TargetPath -WorkingDirectory $link.WorkingDirectory
}
$osc = Start-Process python -ArgumentList "`"$root\eos\osc.py`"", "`"$out\osc.txt`"", "`"$root\eos\$Steps`"" -PassThru -NoNewWindow -RedirectStandardError "$out\osc-err.txt"

$deadline = (Get-Date).AddMinutes(25)
$shots = 0
$viz = $false
while ((Get-Date) -lt $deadline -and -not $osc.HasExited) {
    Start-Sleep 5
    $tree = Snapshot 'eos'
    if (-not $viz) { $viz = Press $tree 'viz' }
    $null = (Press $tree 'I accept|I agree|accept the') -or (Press $tree '^(OK|Continue|Accept)$')
    if (($shots++ % 6) -eq 0) { Shot ('eos-{0:D3}' -f $shots) }
    Get-Process | Where-Object { $_.ProcessName -match 'eos|etc|nomad|augment' } |
        ForEach-Object { "$($_.ProcessName) $($_.Id) $($_.MainWindowTitle)" } | Set-Content "$out\processes.txt"
}
Log "osc steps exited: $($osc.HasExited)"
Start-Sleep 5
Shot 'zz-end'
$null = Snapshot 'end'
pktmon stop | Out-Null
pktmon etl2pcap "$out\eos.etl" --out "$out\eos.pcapng" | Out-Null
Remove-Item "$out\eos.etl" -ErrorAction SilentlyContinue
Stop-Process -Id $listen.Id -ErrorAction SilentlyContinue
Log 'done'
