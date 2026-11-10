# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records demo.mkv of cinq.exe holding its own warnings while the problems of
# its config.toml show on the alternate screen of the terminal.  Two consoles
# stack on the desktop: the top one runs cinq with a show whose video file is
# not there yet, and the bottom one takes commands.  While the problems show,
# the video file arrives, which cinq reads again every two seconds until it
# can play, and its warning about a chapter track that cannot be read is
# held.  Then config.toml is fixed, and the warning comes out on the main
# screen.  Writes the second of each step since the recording started to
# captions.tsv.
#
# Usage: windows.ps1, with cinq.exe, ffmpeg, and the damaged clip in
# clip.mp4 in the working directory

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
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
  // A tap of Alt, which is input of the process's own, after which Windows
  // lets it bring a window to the front
  public static void TapAlt() {
    keybd_event(0x12, 0, 0, UIntPtr.Zero);
    keybd_event(0x12, 0, 2, UIntPtr.Zero);
  }
  // Process and title of each visible window with a title, one to a line
  public static string List() {
    var lines = new System.Collections.Generic.List<string>();
    EnumWindows((window, data) => {
      uint owner;
      GetWindowThreadProcessId(window, out owner);
      var text = new StringBuilder(256);
      GetWindowText(window, text, 256);
      if (IsWindowVisible(window) && text.Length > 0) { lines.Add(owner + "\t" + text); }
      return true;
    }, IntPtr.Zero);
    return string.Join("\n", lines);
  }
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

# Puts the window at the top left corner of the given place, in front
function Place($window, $y, $height) {
  [Windows]::SetWindowPos($window, [IntPtr]::Zero, 0, $y, 1024, $height, 0x40) | Out-Null
}

# Brings the window to the front and waits until it is there, since a window
# that cinq opens takes the focus and Windows refuses to give it back to a
# background process, which only flashes the taskbar button of the window
function Focus($window) {
  foreach ($try in 1..20) {
    [Windows]::TapAlt()
    [Windows]::SetForegroundWindow($window) | Out-Null
    Start-Sleep -Milliseconds 250
    if ([Windows]::GetForegroundWindow() -eq $window) { return }
  }
  Write-Host 'a window did not come to the front'
}

# Moves the window beyond the right edge of the desktop, out of the way
function Hide-Away($window) {
  [Windows]::SetWindowPos($window, [IntPtr]::Zero, 1100, 0, 800, 600, 0x14) | Out-Null
}

# Types a line into the window one key at a time, as a user would
function Type-Into($window, $keys) {
  Focus $window
  $keys.ToCharArray() | ForEach-Object {
    [System.Windows.Forms.SendKeys]::SendWait("$_")
    Start-Sleep -Milliseconds 50
  }
  [System.Windows.Forms.SendKeys]::SendWait("{ENTER}")
}

# Makes the text of a Windows Terminal window larger with its key for it
function Enlarge($window) {
  Focus $window
  1..3 | ForEach-Object {
    [System.Windows.Forms.SendKeys]::SendWait('^=')
    Start-Sleep -Milliseconds 200
  }
}

# Prints the visible windows with the programs that own them
function Show-Windows($when) {
  Write-Host "windows $when"
  [Windows]::List() -split "`n" | ForEach-Object {
    $id, $title = $_ -split "`t", 2
    "  {0} {1}: {2}" -f $id, (Get-Process -Id $id -ErrorAction SilentlyContinue).ProcessName, $title
  }
}

# Appends a caption that starts now
function Caption($text) {
  "{0:F2}`t{1}" -f ((Get-Date) - $script:start).TotalSeconds, $text | Add-Content captions.tsv
}

# Directory of the demo, with a configuration that holds a setting that
# cinq does not know, a show with a video cue whose file is not there, and
# the clip, which arrives later through the directory incoming
$demo = 'C:\demo'
New-Item -ItemType Directory -Force "$demo\conf", "$demo\incoming" | Out-Null
Copy-Item cinq.exe $demo
Copy-Item clip.mp4 "$demo\incoming\clip.mp4"
[IO.File]::WriteAllText("$demo\conf\config.toml", "colour = `"red`"`n")
[IO.File]::WriteAllText("$demo\held.cinq.toml", @"
[[list]]
name = "Main"

[[list.cue]]
id = 1
kind = "video"
name = "Clip"
file = "clip.mp4"
"@)
# Software rendering of cinq's own, since WARP draws slowly
$env:ICED_BACKEND = 'tiny-skia'

# The runner leaves its Performance Options windows open, which would peek out
# from behind the consoles
Stop-Process -Name SystemPropertiesPerformance -Force -ErrorAction SilentlyContinue
(New-Object -ComObject Shell.Application).MinimizeAll()
Start-Sleep 2
# ffmpeg through cmd, which takes its standard error into a file and passes
# its standard input on, so that a q at the end makes ffmpeg finish the file,
# since one that is cut off loses the last seconds; the time limit is only
# for when that fails
$info = New-Object System.Diagnostics.ProcessStartInfo
$info.FileName = 'cmd.exe'
$info.Arguments = '/c ffmpeg -y -f gdigrab -framerate 15 -i desktop -t 90 -pix_fmt yuv420p demo.mkv 2> ffmpeg.txt'
$info.UseShellExecute = $false
$info.RedirectStandardInput = $true
$info.CreateNoWindow = $true
$recorder = [System.Diagnostics.Process]::Start($info)
# The recording starts once ffmpeg says that it waits for a key to stop
foreach ($second in 1..80) {
  $ready = try { Select-String -Path ffmpeg.txt -Pattern 'Press \[q\]' -Quiet } catch { $false }
  if ($ready) { break }
  Start-Sleep -Milliseconds 250
}
$start = Get-Date
Start-Sleep 1

# Two consoles, the top one for cinq and the bottom one for commands, both
# in the directory of the demo
Caption 'cinq runs in the top console, and commands go into the bottom one'
Start-Process cmd.exe -ArgumentList '/k', 'title cinq console' -WorkingDirectory $demo
$run = Wait-Window 0 'cinq console'
Enlarge $run
Place $run 0 480
$setup = '$Host.UI.RawUI.WindowTitle = "commands console"; function prompt { "PS> " }; Clear-Host'
$encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($setup))
Start-Process powershell.exe -ArgumentList '-NoLogo', '-NoExit', '-EncodedCommand', $encoded -WorkingDirectory $demo
$commands = Wait-Window 0 'commands console'
Enlarge $commands
Place $commands 480 240
Start-Sleep 2

Caption 'config.toml sets something that cinq does not know'
Type-Into $commands 'cat conf\config.toml'
Start-Sleep 2

Caption 'cinq starts, and the problem takes the alternate screen of the console'
$before = @(Get-Process cinq -ErrorAction SilentlyContinue).Id
Type-Into $run 'cinq.exe --config-dir conf held.cinq.toml'
$cinq = Wait-New cinq $before
$main = Wait-Window $cinq.Id '.toml - cinq'
Hide-Away $main
# The stage window opens once the show holds a video cue
$stage = Wait-Window $cinq.Id ' stage - cinq'
Hide-Away $stage
Start-Sleep 3
Show-Windows 'after cinq started'

Caption 'The video file arrives, and cinq reads it and warns of a track that it cannot read'
Type-Into $commands 'Move-Item incoming\clip.mp4 clip.mp4'
Start-Sleep 7
Caption 'The warning is held: the problems screen stays clean'
Start-Sleep 4

Caption 'Fixing config.toml: the console switches back, and the held warning comes out'
Type-Into $commands "Set-Content conf\config.toml ''"
Start-Sleep 8
Caption 'cinq goes on, with no problem left to show'
Start-Sleep 4

Show-Windows 'after the fix'
[Windows]::PostMessage($main, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
$cinq.WaitForExit()
Caption ''
Start-Sleep 1
$recorder.StandardInput.Write('q')
$recorder.StandardInput.Flush()
$recorder.WaitForExit(120000) | Out-Null
