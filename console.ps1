# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Checks cinq.exe, a console program whose manifest asks for no console of
# its own: started from Explorer, it should get no console, which a conhost
# child would show, as one does for cmd.exe started the same way; started
# from cmd in a console window, its standard error should show the problems
# of its config.toml there as they change, while cmd waits for it.  Takes a
# screenshot of each step.

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing, System.Windows.Forms
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

function Shoot($file) {
  $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $shot = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
  $graphics = [System.Drawing.Graphics]::FromImage($shot)
  $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
  $shot.Save("$PWD\$file")
  $graphics.Dispose()
  $shot.Dispose()
}

# Process of `name` that was not running before, once it starts
function Wait-New($name, $before) {
  foreach ($second in 1..90) {
    Start-Sleep 1
    $new = Get-Process $name -ErrorAction SilentlyContinue | Where-Object { $before -notcontains $_.Id }
    if ($new) { return @($new)[0] }
  }
  throw "no new $name started"
}

# Window of the process whose title holds part, once it opens, which can be
# late where Defender scans a program first
function Wait-Window($process, $part) {
  foreach ($second in 1..90) {
    Start-Sleep 1
    $window = [Windows]::Find([uint32]$process.Id, $part)
    if ($window -ne [IntPtr]::Zero) { return $window }
  }
  throw "no window of $($process.Name) holds $part"
}

# Consoles that Windows made for the process, each a conhost child of it
function Consoles($process) {
  @(Get-CimInstance Win32_Process -Filter "Name = 'conhost.exe'" | Where-Object ParentProcessId -eq $process.Id).Count
}

$build = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion').CurrentBuild
"Windows build $build"
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
[IO.File]::WriteAllText($config, $broken)
$failed = $false

# Control: a console program without the manifest, which Explorer starts
# with a console of its own
$before = @(Get-Process cmd -ErrorAction SilentlyContinue).Id
Start-Process explorer.exe -ArgumentList "C:\Windows\System32\cmd.exe"
$control = Wait-New cmd $before
Start-Sleep 3
$count = Consoles $control
"cmd.exe started from Explorer: $count console(s)"
if ($count -lt 1) { "the check sees no console even for cmd.exe"; $failed = $true }
Stop-Process $control
Start-Sleep 2

# cinq.exe started from Explorer, as a double click starts it
$before = @(Get-Process cinq -ErrorAction SilentlyContinue).Id
Start-Process explorer.exe -ArgumentList "`"$exe`""
$cinq = Wait-New cinq $before
$main = Wait-Window $cinq " - cinq"
Start-Sleep 3
$parent = (Get-CimInstance Win32_Process -Filter "ProcessId = $($cinq.Id)").ParentProcessId
$started = (Get-CimInstance Win32_Process -Filter "ProcessId = $parent").Name
$count = Consoles $cinq
"cinq.exe started from Explorer, by $started`: $count console(s)"
if ($count -gt 0) { $failed = $true }
[Windows]::SetWindowPos($main, [IntPtr]::Zero, 0, 0, 1024, 740, 0x40) | Out-Null
Start-Sleep 2
Shoot "1-explorer.png"
Stop-Process $cinq
Start-Sleep 2

# cinq.exe typed into cmd in a console window, with tiny-skia, since WARP
# draws slowly
$env:ICED_BACKEND = 'tiny-skia'
$before = @(Get-Process cinq -ErrorAction SilentlyContinue).Id
Start-Process cmd.exe -ArgumentList '/k', 'mode con: cols=62 lines=46 && title cinq console' -WorkingDirectory $PWD
$console = [IntPtr]::Zero
foreach ($second in 1..30) {
  Start-Sleep 1
  $console = [Windows]::Find(0, "cinq console")
  if ($console -ne [IntPtr]::Zero) { break }
}
[Windows]::SetWindowPos($console, [IntPtr]::Zero, 0, 0, 0, 0, 0x41) | Out-Null
[Windows]::SetForegroundWindow($console) | Out-Null
Start-Sleep 1
[System.Windows.Forms.SendKeys]::SendWait("cinq.exe{ENTER}")
$cinq = Wait-New cinq $before
$main = Wait-Window $cinq " - cinq"
Start-Sleep 3
[Windows]::SetWindowPos($main, [IntPtr]::Zero, 520, 0, 504, 740, 0x40) | Out-Null
Start-Sleep 2
"cinq.exe typed into cmd: $(Consoles $cinq) console(s) of its own"
Shoot "2-console.png"
[IO.File]::WriteAllText($config, $broken.Replace("fly", "help"))
Start-Sleep 3
Shoot "3-one-left.png"
[IO.File]::WriteAllText($config, $broken.Replace("fly", "help").Replace("Foo", "PageDown"))
Start-Sleep 3
Shoot "4-fixed.png"
[IO.File]::WriteAllText($config, $broken.Replace("fly", "help").Replace("Foo", "PageDown").TrimEnd().TrimEnd("]"))
Start-Sleep 3
Shoot "5-not-toml.png"
Stop-Process $cinq
Start-Sleep 3
Shoot "6-closed.png"

if ($failed) { exit 1 }
