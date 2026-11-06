# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Walks through show\pictures.cinq.toml by the steps of steps.txt, with
# cinq's main window and stage side by side until a cue makes the stage
# fullscreen, and records the desktop

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

function Shoot($file) {
  $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $shot = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
  $graphics = [System.Drawing.Graphics]::FromImage($shot)
  $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
  $shot.Save("$PWD\$file")
  $graphics.Dispose()
  $shot.Dispose()
}

# Prompt to update WSL, which windows-11-arm opens on its own in Windows
# Terminal, where it takes the keys meant for cinq
function Close-Prompt {
  Stop-Process -Name wsl -Force -ErrorAction SilentlyContinue
  [Windows]::CloseTitled("wsl.exe")
}

# Keys of the actions of steps.txt, as SendKeys writes them
$keys = @{ go = " "; fullscreen = "{F11}"; panic = "{ESC}" }

# Presses the key of an action in whichever window of cinq has the
# keyboard, since keys act on the cue list from either, and gives the
# keyboard to the stage only when another program has taken it, so that
# the main window never covers the stage as the operator would not
function Press($action) {
  Close-Prompt
  $front = [Windows]::GetForegroundWindow()
  if ($front -ne $main -and $front -ne $stage) {
    [Windows]::SetForegroundWindow($stage) | Out-Null
    Start-Sleep -Milliseconds 300
  }
  [System.Windows.Forms.SendKeys]::SendWait($keys[$action])
}

# Windows of the first logon, which cover the desktop of windows-11-arm
if (Get-Process WWAHost -ErrorAction SilentlyContinue) {
  Stop-Process -Name WWAHost -Force
  Start-Sleep 2
  [System.Windows.Forms.SendKeys]::SendWait("{ESC}")
}
Close-Prompt

$cinq = Start-Process .\cinq.exe -ArgumentList "pictures.cinq.toml" -WorkingDirectory "$PWD\show" -RedirectStandardError "$PWD\err.txt" -PassThru
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
Close-Prompt
"main $main, stage $stage"
# Main window and stage side by side on the desktop of 1024x768
[Windows]::SetWindowPos($main, [IntPtr]::Zero, 0, 0, 560, 740, 0x40) | Out-Null
[Windows]::SetWindowPos($stage, [IntPtr]::Zero, 560, 0, 464, 300, 0x40) | Out-Null
[Windows]::SetForegroundWindow($main) | Out-Null
Start-Sleep 2
Shoot "opened.png"
$recorder = Start-Process ffmpeg -ArgumentList "-loglevel error -y -f gdigrab -framerate 15 -t 34 -i desktop -pix_fmt yuv420p pictures.mp4" -NoNewWindow -PassThru
Start-Sleep 2
Get-Content steps.txt | Where-Object { $_ -match '^\d' } | ForEach-Object {
  $delay, $action = -split $_
  Start-Sleep $delay
  if ($keys.ContainsKey($action)) { Press $action }
  Shoot "step-$($_ -replace ' ', '-').png"
}
$recorder.WaitForExit()
"--- stderr"
Get-Content err.txt
$running = -not $cinq.HasExited
if ($running) { Stop-Process $cinq } else { "cinq exited with $($cinq.ExitCode)"; exit 1 }
