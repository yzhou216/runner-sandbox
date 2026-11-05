# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays the video cue of show\video.cinq.toml with cinq's main window and
# stage side by side, records the desktop, and then fades the cue out

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

function Shoot($file) {
  $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $shot = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
  $graphics = [System.Drawing.Graphics]::FromImage($shot)
  $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
  $shot.Save("$PWD\$file")
  $graphics.Dispose()
  $shot.Dispose()
}

function Go($window) {
  [Windows]::SetForegroundWindow($window) | Out-Null
  Start-Sleep -Milliseconds 300
  [System.Windows.Forms.SendKeys]::SendWait(" ")
}

# Windows of the first logon, which cover the desktop of windows-11-arm
if (Get-Process WWAHost -ErrorAction SilentlyContinue) {
  Stop-Process -Name WWAHost -Force
  Start-Sleep 2
  [System.Windows.Forms.SendKeys]::SendWait("{ESC}")
}
Stop-Process -Name wsl -Force -ErrorAction SilentlyContinue

$cinq = Start-Process .\cinq.exe -ArgumentList "video.cinq.toml" -WorkingDirectory "$PWD\show" -RedirectStandardError "$PWD\err.txt" -PassThru
# Windows of cinq, which open late where Defender scans a program first
$main = [IntPtr]::Zero
$stage = [IntPtr]::Zero
foreach ($second in 1..60) {
  Start-Sleep 1
  $main = [Windows]::Find($cinq.Id, ".toml - cinq")
  $stage = [Windows]::Find($cinq.Id, " stage - cinq")
  if ($main -ne [IntPtr]::Zero -and $stage -ne [IntPtr]::Zero) { break }
}
Start-Sleep 3
"main $main, stage $stage"
# Main window and stage side by side on the desktop of 1024x768
[Windows]::SetWindowPos($main, [IntPtr]::Zero, 0, 0, 600, 740, 0x40) | Out-Null
[Windows]::SetWindowPos($stage, [IntPtr]::Zero, 600, 0, 424, 300, 0x40) | Out-Null
Start-Sleep 2
Shoot "1-opened.png"
$recorder = Start-Process ffmpeg -ArgumentList "-loglevel error -y -f gdigrab -framerate 15 -t 16 -i desktop -pix_fmt yuv420p video.mp4" -NoNewWindow -PassThru
Start-Sleep 2
Go $main
Start-Sleep 5
Shoot "2-playing.png"
Go $main
Start-Sleep 1
Shoot "3-fading.png"
Start-Sleep 4
Shoot "4-faded.png"
$recorder.WaitForExit()
"--- stderr"
Get-Content err.txt
$running = -not $cinq.HasExited
if ($running) { Stop-Process $cinq } else { "cinq exited with $($cinq.ExitCode)"; exit 1 }
