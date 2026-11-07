# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays show\video.cinq.toml in cinq.exe as play.sh does on GNU/Linux: GO
# starts a chord, and GO fades it out, after which the video follows, whose
# picture shows its own time.  Takes a screenshot at each step and writes
# the time of each since cinq started to steps.txt, without a recording of
# the sound, which Windows runners have no device for.

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
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr window, uint message, IntPtr wparam, IntPtr lparam);
  // Closes the visible windows whose title holds part, as the prompt of WSL
  public static void CloseTitled(string part) {
    EnumWindows((window, data) => {
      var text = new StringBuilder(256);
      GetWindowText(window, text, 256);
      if (IsWindowVisible(window) && text.ToString().Contains(part)) { PostMessage(window, 0x10, IntPtr.Zero, IntPtr.Zero); }
      return true;
    }, IntPtr.Zero);
  }
  public static IntPtr Find(uint process, string suffix) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((window, data) => {
      uint owner;
      GetWindowThreadProcessId(window, out owner);
      var text = new StringBuilder(256);
      GetWindowText(window, text, 256);
      if (owner == process && IsWindowVisible(window) && text.ToString().EndsWith(suffix)) { found = window; return false; }
      return true;
    }, IntPtr.Zero);
    return found;
  }
}
"@

New-Item -ItemType Directory -Force out | Out-Null
$clock = [System.Diagnostics.Stopwatch]::StartNew()

# Notes a step with the seconds since cinq started
function Mark($step) {
  $line = "{0,6:N2} s  {1}" -f $clock.Elapsed.TotalSeconds, $step
  $line
  Add-Content out\steps.txt $line
}

function Shoot($name) {
  $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $shot = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
  $graphics = [System.Drawing.Graphics]::FromImage($shot)
  $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
  $shot.Save("$PWD\out\$name.png")
  $graphics.Dispose()
  $shot.Dispose()
  Mark "shot $name"
}

# Prompt to update WSL, which windows-11-arm opens on its own in Windows
# Terminal, where it takes the keys meant for cinq
function Close-Prompt {
  Stop-Process -Name wsl -Force -ErrorAction SilentlyContinue
  [Windows]::CloseTitled("wsl.exe")
}

# Presses GO in whichever window of cinq has the keyboard, since keys act
# on the cue list from either, giving the keyboard back to cinq only when
# another program has taken it
function Go {
  Close-Prompt
  $front = [Windows]::GetForegroundWindow()
  if ($front -ne $main -and $front -ne $stage) {
    [Windows]::SetForegroundWindow($main) | Out-Null
    Start-Sleep -Milliseconds 300
  }
  [System.Windows.Forms.SendKeys]::SendWait(" ")
  Mark "key space"
}

# Windows of the first logon, which cover the desktop of windows-11-arm
if (Get-Process WWAHost -ErrorAction SilentlyContinue) {
  Stop-Process -Name WWAHost -Force
  Start-Sleep 2
  [System.Windows.Forms.SendKeys]::SendWait("{ESC}")
}
Close-Prompt

$clock.Restart()
$cinq = Start-Process .\cinq.exe -ArgumentList "video.cinq.toml" -WorkingDirectory "$PWD\show" -RedirectStandardError "$PWD\out\err.txt" -PassThru
# Windows of cinq, which open late where Defender scans a program first
$main = [IntPtr]::Zero
$stage = [IntPtr]::Zero
foreach ($second in 1..120) {
  Start-Sleep -Milliseconds 500
  $main = [Windows]::Find($cinq.Id, ".toml - cinq")
  $stage = [Windows]::Find($cinq.Id, " stage - cinq")
  if ($main -ne [IntPtr]::Zero -and $stage -ne [IntPtr]::Zero) { break }
}
Mark "windows of cinq: main $main, stage $stage"
Start-Sleep 3
Close-Prompt
# Main window and stage side by side on the desktop of 1024x768
[Windows]::SetWindowPos($main, [IntPtr]::Zero, 0, 0, 560, 740, 0x40) | Out-Null
[Windows]::SetWindowPos($stage, [IntPtr]::Zero, 560, 0, 464, 300, 0x40) | Out-Null
[Windows]::SetForegroundWindow($main) | Out-Null
Start-Sleep 2
Shoot "opened"
Go
Start-Sleep 4
Shoot "chord"
Go
Start-Sleep 1
Shoot "fading"
foreach ($number in 1..6) {
  Start-Sleep -Milliseconds 1500
  Shoot "video-$number"
}
Start-Sleep 3
Shoot "ended"
$running = -not $cinq.HasExited
"cinq still running: $running"
Stop-Process -Id $cinq.Id -Force -ErrorAction SilentlyContinue
"--- stderr"
Get-Content out\err.txt
if (-not $running) { exit 1 }
