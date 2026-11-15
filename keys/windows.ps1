# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Drives cinq's vi and cua keys on Windows with SendInput, as the keyboard
# does, and the back and forward buttons of the mouse with SendInput of
# XBUTTON1 and XBUTTON2.  cinq's standard error goes to out\vi.err and
# out\cua.err, and each step writes the time and its name to out\vi.marks or
# out\cua.marks, so that verdict.sh can put the lines of cinq between the
# marks of the keys that caused them.  cinq.exe comes from cinq-windows\bin in
# the checkout.
#
# Usage: windows.ps1

. "$PSScriptRoot\ui.ps1"

$exe = "$root\cinq-windows\bin\cinq.exe"
$env:ICED_BACKEND = 'tiny-skia'

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Threading;
public static class Native {
    [StructLayout(LayoutKind.Sequential)] struct MOUSEINPUT { public int dx, dy; public uint mouseData, flags, time; public IntPtr extra; }
    [StructLayout(LayoutKind.Sequential)] struct KEYBDINPUT { public ushort vk, scan; public uint flags, time; public IntPtr extra; }
    [StructLayout(LayoutKind.Explicit)] struct Union {
        [FieldOffset(0)] public MOUSEINPUT mi;
        [FieldOffset(0)] public KEYBDINPUT ki;
    }
    [StructLayout(LayoutKind.Sequential)] struct INPUT { public uint type; public Union u; }
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int left, top, right, bottom; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] inputs, int size);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hh, bool repaint);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);

    static uint Send(INPUT input) { return SendInput(1, new[] { input }, Marshal.SizeOf(typeof(INPUT))); }

    static void Key(ushort vk, bool up) {
        // Arrows, End, and Page Down are the extended keys of the gray block
        bool extended = vk >= 0x21 && vk <= 0x28;
        var input = new INPUT { type = 1 };
        input.u.ki = new KEYBDINPUT { vk = vk, flags = (extended ? 1u : 0u) | (up ? 2u : 0u) };
        Send(input);
    }

    // Presses KEY with the modifiers held, in order, and lets go in reverse
    public static void Combo(ushort vk, ushort[] mods) {
        foreach (var m in mods) { Key(m, false); Thread.Sleep(40); }
        Key(vk, false);
        Thread.Sleep(40);
        Key(vk, true);
        Thread.Sleep(40);
        for (int i = mods.Length - 1; i >= 0; i--) { Key(mods[i], true); Thread.Sleep(40); }
    }

    // Presses and releases XBUTTON1 (back) or XBUTTON2 (forward)
    public static uint XButton(uint which) {
        var down = new INPUT { type = 0 };
        down.u.mi = new MOUSEINPUT { mouseData = which, flags = 0x0080 };
        uint a = Send(down);
        Thread.Sleep(60);
        var up = new INPUT { type = 0 };
        up.u.mi = new MOUSEINPUT { mouseData = which, flags = 0x0100 };
        return a + Send(up);
    }
}
'@

$vk = @{ ctrl = 0x11; alt = 0x12; shift = 0x10; left = 0x25; right = 0x27; end = 0x23; pgdn = 0x22
    tab = 0x09; 'return' = 0x0D; f1 = 0x70; slash = 0xBF }
foreach ($c in [char[]]'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789') { $vk[[string]$c] = [int]$c }

$script:phase = ''
$script:window = $null
$script:cinq = $null

# Counts the lines of cinq's standard error that match PATTERN, which cinq still
# holds open for writing
function Count-Log([string]$pattern) {
    try {
        $f = [IO.File]::Open("$out\$script:phase.err", 'Open', 'Read', 'ReadWrite,Delete')
        $r = New-Object IO.StreamReader($f)
        $n = 0
        while (($line = $r.ReadLine()) -ne $null) { if ($line -match $pattern) { $n++ } }
        $r.Dispose()
        $n
    } catch { 0 }
}

# Waits until cinq has logged COUNT lines that match PATTERN, since a runner
# that draws slowly, such as windows-11-arm, handles a key long after it came
function Wait-Log([string]$pattern, [int]$count, [int]$seconds = 40) {
    $end = (Get-Date).AddSeconds($seconds)
    while ((Count-Log $pattern) -lt $count -and (Get-Date) -lt $end) { Start-Sleep -Milliseconds 250 }
    if ((Count-Log $pattern) -lt $count) { Log "timeout: fewer than $count lines of $pattern" }
}

$script:buttons = 0

function Step([string]$label) {
    Add-Content "$out\$script:phase.marks" ('{0:F3} MARK {1}' -f (Epoch), $label)
    Log "step $label"
}

function Main-Window {
    Get-Process -Id $script:cinq.Id -ErrorAction SilentlyContinue | Where-Object MainWindowHandle -ne 0 | Select-Object -First 1
}

# Sends one key, as keys like 'ctrl+o', 'shift+g', 'alt+left', or 'l', in
# the foreground window after the update window of the subsystem is closed,
# then waits until cinq has logged one more line that matches PATTERN for each
# key.  The keys that the keymap reads log a press, while those typed into the
# search line or the menu of a drop-down list log what the call's pattern names.
function Key([string]$label, [string[]]$specs, [string]$pattern = ' press Press ') {
    CloseWsl
    $w = Main-Window
    if (-not $w) { Log 'no cinq window'; return }
    $null = [Native]::SetForegroundWindow($w.MainWindowHandle)
    Start-Sleep -Milliseconds 400
    $before = Count-Log $pattern
    Step $label
    foreach ($spec in $specs) {
        $names = $spec.ToLower() -split '\+'
        $mods = @()
        if ($names.Count -gt 1) { $mods = @($names[0..($names.Count - 2)] | ForEach-Object { [uint16]$vk[$_] }) }
        [Native]::Combo([uint16]$vk[$names[-1]], [uint16[]]$mods)
        Start-Sleep -Milliseconds 250
    }
    Wait-Log $pattern ($before + $specs.Count)
    Start-Sleep -Seconds 1
}

function Button([string]$label, [uint32]$which) {
    CloseWsl
    $w = Main-Window
    if (-not $w) { Log 'no cinq window'; return }
    $null = [Native]::SetForegroundWindow($w.MainWindowHandle)
    $r = New-Object Native+RECT
    $null = [Native]::GetWindowRect($w.MainWindowHandle, [ref]$r)
    $null = [Native]::SetCursorPos([int](($r.left + $r.right) / 2), [int](($r.top + $r.bottom) / 2))
    Start-Sleep -Milliseconds 500
    Step $label
    $sent = [Native]::XButton($which)
    Log "SendInput XBUTTON$which returned $sent"
    $script:buttons++
    Wait-Log ' mouse button pressed ' $script:buttons
    Start-Sleep -Seconds 1
}

function Start-Cinq([string]$name, [string]$keymap, [string]$show) {
    $script:phase = $name
    $script:buttons = 0
    $config = Join-Path $env:TEMP "cinq-$name"
    New-Item -ItemType Directory -Force -Path $config | Out-Null
    Copy-Item "$PSScriptRoot\$keymap.toml" "$config\config.toml"
    Copy-Item "$PSScriptRoot\$show.cinq.toml" "$config\show.cinq.toml"
    CloseWsl
    Log "starting $name"
    $script:cinq = Start-Process $exe -ArgumentList '--config-dir', "`"$config`"", "`"$config\show.cinq.toml`"" -PassThru `
        -RedirectStandardOutput "$out\$name.out" -RedirectStandardError "$out\$name.err"
    $window = $null
    for ($i = 0; $i -lt 60 -and -not $window; $i++) {
        Start-Sleep 1
        $window = Main-Window
    }
    if (-not $window) { Log 'no cinq window'; Shot "$name-none"; exit 1 }
    # Window of 900 by 640, so that the list shows fewer rows than the
    # window could
    $null = [Native]::MoveWindow($window.MainWindowHandle, 60, 40, 900, 640, $true)
    Start-Sleep 4
    Shot "$name-0-start"
}

function Stop-Cinq { Stop-Process -Id $script:cinq.Id -Force -ErrorAction SilentlyContinue; Start-Sleep 2 }

Mark "computer $env:COMPUTERNAME arch $env:PROCESSOR_ARCHITECTURE"
(Get-CimInstance Win32_OperatingSystem | ForEach-Object { "$($_.Caption) $($_.Version) $($_.OSArchitecture)" }) | Set-Content "$out\os.txt"
Add-Content "$out\os.txt" "mouse buttons: $([System.Windows.Forms.SystemInformation]::MouseButtons) present $([System.Windows.Forms.SystemInformation]::MousePresent)"

Start-Cinq 'vi' 'vi' 'show'
# 1. Control+i after Control+o
Key '1-l' 'l'
Key '1-G' 'shift+g'
Key '1-ctrl-o' 'ctrl+o'
Key '1-ctrl-i' 'ctrl+i'
Shot 'vi-1-back-forward'
# 3. Half pages
Key '3-gg' 'g','g'
Key '3-ctrl-d' 'ctrl+d'
Shot 'vi-3-ctrl-d'
Key '3-ctrl-u' 'ctrl+u'
# 4. Copy, paste, cut, undo
Key '4-gg' 'g','g'
Key '4-yy' 'y','y'
Key '4-p' 'p'; Key '4-p' 'p'; Key '4-p' 'p'
Shot 'vi-4-pasted'
Key '4-dd' 'd','d'
Key '4-u' 'u'
Key '4-3u' '3','u'
Shot 'vi-4-undone'
# 5. Control+o opens nothing, Control+l opens the file dialog
Key '5-ctrl-o' 'ctrl+o'
Shot 'vi-5-after-ctrl-o'
Key '5-ctrl-l' 'ctrl+l'
$tree = $null
for ($i = 0; $i -lt 12; $i++) {
    Start-Sleep 3
    $tree = Snapshot 'vi-5-dialog'
    if ($tree | Where-Object { $_.Kind -eq 'Button' -and $_.Name -eq 'Cancel' }) { break }
}
($tree | ForEach-Object Text) -join "`n" | Set-Content "$out\vi-5-windows.txt"
Step '5-cancel'
$pressed = Press $tree '^Cancel$' 'Button'
Log "Cancel pressed: $pressed"
Start-Sleep 2
(Windows | ForEach-Object Text) -join "`n" | Set-Content "$out\vi-5-windows-after.txt"
Shot 'vi-5-after-cancel'
# 6. Buttons of the mouse
Key '6-gg' 'g','g'
Key '6-l' 'l'
Key '6-G' 'shift+g'
Button '6-xbutton1-back' 1
Button '6-xbutton2-forward' 2
Shot 'vi-6-buttons'
Stop-Cinq

Start-Cinq 'cua' 'cua' 'show'
# 2. Alt with the arrows
Key '2-ctrl-pagedown' 'ctrl+pgdn'
Key '2-end' 'end'
Key '2-alt-left' 'alt+left'
Key '2-alt-right' 'alt+right'
Shot 'cua-2-alt-arrows'
Stop-Cinq

$searching = ' message Searching\('
$accepted = ' message SearchAccepted'
$menuKey = ' menu key='
$opened = ' message MenuOpen\(true'

# 7. Operators: d 2 j cuts three cues, p pastes them below
Start-Cinq 'ops' 'vi' 'long'
Key '7-gg' 'g','g'
Key '7-d2j' 'd','2','j'
Shot 'ops-7-cut'
Key '7-p' 'p'
Shot 'ops-7-pasted'
# 11. A stray key after d is dropped: d u does not undo
Key '11-du' 'd','u'
Shot 'ops-11-du'
Stop-Cinq

# 8. Search, forward and backward, with n, N, and Control+o
Start-Cinq 'search' 'vi' 'long'
Key '8-gg' 'g','g'
Key '8-slash' 'slash'
Key '8-text' '1' $searching
Key '8-enter' 'return' $accepted
Key '8-n1' 'n'
Key '8-n2' 'n'
Key '8-qmark' 'shift+slash'
Key '8-text-back' '1' $searching
Key '8-enter-back' 'return' $accepted
Key '8-n3' 'n'
Key '8-N' 'shift+n'
Key '8-ctrl-o' 'ctrl+o'
Key '8-9' '9'
Key '8-9-ctrl-o' 'ctrl+o'
Shot 'search-8-back'
# 12. F1 shows the help page, which q closes
Key '12-f1' 'f1'
Shot 'search-12-help'
Key '12-q' 'q'
Stop-Cinq

# 9. View keys
Start-Cinq 'view' 'vi' 'long'
Key '9-gg' 'g','g'
Key '9-ctrl-e-1' 'ctrl+e'
Key '9-ctrl-e-2' 'ctrl+e'
Key '9-ctrl-e-3' 'ctrl+e'
Shot 'view-9-scrolled'
Key '9-ctrl-y' 'ctrl+y'
Key '9-H' 'shift+h'
Key '9-L' 'shift+l'
Key '9-zz' 'z','z'
Shot 'view-9-zz'
Key '9-zt' 'z','t'
Shot 'view-9-zt'
Key '9-H-after-zt' 'shift+h'
Stop-Cinq

# 10. A drop-down list: Tab to it, Enter opens it, Control+y picks, Control+e closes
Start-Cinq 'menu' 'vi' 'show'
Key '10-tab' 'tab'
Shot 'menu-10-focused'
Key '10-enter' 'return' $opened
Shot 'menu-10-open'
Key '10-j' 'j' $menuKey
Key '10-ctrl-y' 'ctrl+y' $menuKey
Shot 'menu-10-picked'
Key '10-tab-again' 'tab'
Key '10-enter-again' 'return' $opened
Key '10-ctrl-e' 'ctrl+e' $menuKey
Shot 'menu-10-closed'
Stop-Cinq
Mark 'done'
