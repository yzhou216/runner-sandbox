// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// Opens an NSOpenPanel the way rfd does, in the directory that the first
// argument names, with the allowed file types that the others name, and
// prints the path chosen or "cancelled".
import AppKit

let arguments = CommandLine.arguments
let directory = URL(fileURLWithPath: arguments[1], isDirectory: true)
let types = Array(arguments.dropFirst(2))

let application = NSApplication.shared
application.setActivationPolicy(.regular)
application.activate(ignoringOtherApps: true)

let panel = NSOpenPanel()
if !types.isEmpty {
    panel.allowedFileTypes = types
}
panel.directoryURL = directory
panel.canChooseDirectories = false
panel.canChooseFiles = true

if panel.runModal() == .OK, let url = panel.url {
    print(url.path)
} else {
    print("cancelled")
}
