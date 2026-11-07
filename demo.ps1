# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records demo.mkv of the indicatif prototype of cinq.exe, typed into cmd
# with its log on: the log lines stay above, and the problems of the latest
# reading of config.toml stay at the bottom.  The script rewrites the file at
# each step, closes cinq and starts it again, and at the end narrows the
# terminal.  Writes the time of each step since the recording started to
# captions.tsv.

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

# cinq.exe typed into the console, once its main window opens beside it
function Start-Cinq($console) {
  $before = @(Get-Process cinq -ErrorAction SilentlyContinue).Id
  Type-Into $console "cinq.exe"
  $cinq = Wait-New cinq $before
  $main = Wait-Window $cinq.Id " - cinq"
  Place $main 600 424
  return $cinq, $main
}

function Close-Cinq($cinq, $main) {
  [Windows]::PostMessage($main, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
  $cinq.WaitForExit()
}

$start = Get-Date
function Caption($text) {
  "{0:F1}`t{1}" -f ((Get-Date) - $start).TotalSeconds, $text | Add-Content captions.tsv
}

$config = Join-Path $env:APPDATA 'cinq\config.toml'
New-Item -ItemType Directory -Force (Split-Path $config) | Out-Null
$broken = @"
keymap = "booth"

[keymaps.booth]
from = "cua"
go = ["Space", "F5"]
fly = "f"

[keys]
next = ["Foo", "ArrowDown"]
"@
$one = $broken.Replace("fly", "help")
$fixed = $one.Replace("Foo", "PageDown")
[IO.File]::WriteAllText($config, $broken)
# Software rendering of cinq's own, since WARP draws slowly
$env:ICED_BACKEND = 'tiny-skia'
$build = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion').CurrentBuild
"Windows build $build"

(New-Object -ComObject Shell.Application).MinimizeAll()
Start-Sleep 2
Start-Process cmd.exe -ArgumentList '/k', 'title cinq console' -WorkingDirectory $PWD
$console = Wait-Window 0 "cinq console"
Place $console 0 600
Start-Sleep 2
$recorder = Start-Process ffmpeg -PassThru -NoNewWindow -RedirectStandardError ffmpeg.txt -ArgumentList '-y', '-f', 'gdigrab', '-framerate', '15', '-i', 'desktop', '-t', '150', '-pix_fmt', 'yuv420p', 'demo.mkv'
$start = Get-Date
Start-Sleep 2

Caption "config.toml has two mistakes, and cinq starts from cmd with its log on"
Type-Into $console "set RUST_LOG=cinq=info"
$cinq, $main = Start-Cinq $console
Caption "The log line of each reading stays on top, and its problems stay at the bottom"
Start-Sleep 8
Caption "Fixing the action: the next log line goes above, and the problem left takes the place of both"
[IO.File]::WriteAllText($config, $one)
Start-Sleep 7
Caption "Fixing the key: the problems leave, and the prompt and the log stay"
[IO.File]::WriteAllText($config, $fixed)
Start-Sleep 7
Caption "Deleting a closing bracket: the file is no longer TOML"
[IO.File]::WriteAllText($config, $fixed.TrimEnd().TrimEnd("]"))
Start-Sleep 7
Caption "Putting the bracket back: the problems leave again"
[IO.File]::WriteAllText($config, $fixed)
Start-Sleep 6
Caption "Making a mistake again, then closing cinq: the report stays, with the prompt below it"
[IO.File]::WriteAllText($config, $fixed.Replace("help", "fly"))
Start-Sleep 5
Close-Cinq $cinq $main
Start-Sleep 5
Caption "Starting cinq again, which shows the problem below a new log line"
$cinq, $main = Start-Cinq $console
Start-Sleep 7
Caption "Narrowing the terminal: the lines that indicatif padded to the old width wrap again"
Place $console 0 420
Start-Sleep 6
Caption "Fixing the action again: the redraw leaves pieces of the old report behind"
[IO.File]::WriteAllText($config, $fixed)
Start-Sleep 7
Caption "end"
Close-Cinq $cinq $main
$recorder.WaitForExit(200000) | Out-Null
