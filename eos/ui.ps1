# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Helpers that the steps of the Eos workflow share: a log with the time
# since the step started, screenshots, and the windows of the desktop as UI
# Automation sees them, whose buttons can be pressed by name.  Dot-source it.

$ErrorActionPreference = 'Continue'
$root = (Resolve-Path "$PSScriptRoot\..").Path
$out = Join-Path $root 'out'
New-Item -ItemType Directory -Force -Path $out, "$out\shots", "$out\uia" | Out-Null
$start = Get-Date
$shell = New-Object -ComObject WScript.Shell

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
    Log "press $($hit.Kind) '$($hit.Name)'"
    Activate $hit
}

# Presses an element through UI Automation, or clicks its middle where it
# offers no way to
function Activate($hit) {
    $e = $hit.Element
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

