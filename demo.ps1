# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records demo.mkv of cinq.exe, a console program whose manifest asks for no
# console of its own: opened from Explorer, it shows no console window; typed
# into cmd, it prints there, redraws the problems of its config.toml as the
# file changes, and keeps cmd waiting until it closes.  Writes the time of
# each step since the recording started to captions.tsv.

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class Windows {
  public delegate bool EnumProc(IntPtr window, IntPtr data);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc proc, IntPtr data);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr window, StringBuilder text, int length);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr window, uint message, IntPtr w, IntPtr l);
  // Visible window whose title holds part, of the process if one is given
  public static IntPtr Find(uint process, string part) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((window, data) => {
      uint owner;
      GetWindowThreadProcessId(window, out owner);
      var text = new StringBuilder(256);
      GetWindowText(window, text, 256);
      if ((process == 0 || owner == process) && IsWindowVisible(window) && text.ToString().Contains(part)) { found = window; return false; }
      return true;
    }, IntPtr.Zero);
    return found;
  }
}
"@

# Window whose title holds part, of the process if one is given, once it opens
function Wait-Window($process, $part) {
  foreach ($second in 1..90) {
    $window = [Windows]::Find([uint32]$process, $part)
    if ($window -ne [IntPtr]::Zero) { return $window }
    Start-Sleep 1
  }
  throw "no window holds $part"
}

# Process of `name` that was not running before, once it starts
function Wait-New($name, $before) {
  foreach ($second in 1..90) {
    $new = Get-Process $name -ErrorAction SilentlyContinue | Where-Object { $before -notcontains $_.Id }
    if ($new) { return @($new)[0] }
    Start-Sleep 1
  }
  throw "no new $name started"
}

function Place($window, $x, $width) {
  [Windows]::SetWindowPos($window, [IntPtr]::Zero, $x, 0, $width, 720, 0x40) | Out-Null
}

function Type-Into($window, $keys) {
  [Windows]::SetForegroundWindow($window) | Out-Null
  Start-Sleep -Milliseconds 300
  $keys.ToCharArray() | ForEach-Object {
    [System.Windows.Forms.SendKeys]::SendWait("$_")
    Start-Sleep -Milliseconds 70
  }
  [System.Windows.Forms.SendKeys]::SendWait("{ENTER}")
}

$start = Get-Date
function Caption($text) {
  "{0:F1}`t{1}" -f ((Get-Date) - $start).TotalSeconds, $text | Add-Content captions.tsv
}

$exe = (Resolve-Path cinq.exe).Path
$config = Join-Path $env:APPDATA 'cinq\config.toml'
New-Item -ItemType Directory -Force (Split-Path $config) | Out-Null
$broken = @"
keymap = "booth"

[keymaps.booth]
from = "cua"
fly = "f"

[keys]
next = ["Foo", "ArrowDown"]
"@
$fixed = $broken.Replace("fly", "help").Replace("Foo", "PageDown")
[IO.File]::WriteAllText($config, $broken)
# Software rendering of cinq's own, since WARP draws slowly
$env:ICED_BACKEND = 'tiny-skia'

(New-Object -ComObject Shell.Application).MinimizeAll()
Start-Sleep 2
$recorder = Start-Process ffmpeg -PassThru -NoNewWindow -RedirectStandardError ffmpeg.txt -ArgumentList '-y', '-f', 'gdigrab', '-framerate', '15', '-i', 'desktop', '-t', '150', '-pix_fmt', 'yuv420p', 'demo.mkv'
$start = Get-Date
Start-Sleep 2

# Explorer, showing cinq.exe, starts it as a double click does
Caption "Explorer starts cinq.exe, as a double click does"
Start-Process explorer.exe -ArgumentList "/select,`"$exe`""
$folder = Wait-Window 0 (Split-Path -Leaf $PWD)
Place $folder 0 520
Start-Sleep 2
$before = @(Get-Process cinq -ErrorAction SilentlyContinue).Id
Start-Process explorer.exe -ArgumentList "`"$exe`""
$cinq = Wait-New cinq $before
$main = Wait-Window $cinq.Id " - cinq"
Place $main 520 504
$consoles = @(Get-CimInstance Win32_Process -Filter "Name = 'conhost.exe'" | Where-Object ParentProcessId -eq $cinq.Id).Count
"cinq.exe from Explorer: $consoles console(s)"
Caption "No console window beside cinq ($consoles conhost.exe)"
Start-Sleep 6
[Windows]::PostMessage($main, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
[Windows]::PostMessage($folder, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
$cinq.WaitForExit()
Start-Sleep 2

# cinq.exe typed into cmd
Caption "Typed into cmd, cinq prints there"
Start-Process cmd.exe -ArgumentList '/k', 'title cinq console' -WorkingDirectory $PWD
$console = Wait-Window 0 "cinq console"
Place $console 0 520
Start-Sleep 2
Type-Into $console "cinq.exe --version"
Start-Sleep 2
$before = @(Get-Process cinq -ErrorAction SilentlyContinue).Id
Type-Into $console "cinq.exe"
$cinq = Wait-New cinq $before
$main = Wait-Window $cinq.Id " - cinq"
Place $main 520 504
Caption "Problems of config.toml on standard error"
Start-Sleep 7

Caption "config.toml saved: fly renamed help"
[IO.File]::WriteAllText($config, $broken.Replace("fly", "help"))
Start-Sleep 6
Caption "config.toml saved: Foo replaced by PageDown, nothing printed"
[IO.File]::WriteAllText($config, $fixed)
Start-Sleep 6
Caption "config.toml saved without its closing bracket"
[IO.File]::WriteAllText($config, $fixed.TrimEnd().TrimEnd("]"))
Start-Sleep 7
Caption "Bracket back, the screen clears"
[IO.File]::WriteAllText($config, $fixed)
Start-Sleep 6

Caption "cmd waits until cinq closes"
[Windows]::PostMessage($main, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
$cinq.WaitForExit()
Start-Sleep 5
Caption "end"
$recorder.WaitForExit(200000) | Out-Null
