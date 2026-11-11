// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// Drives the screen by the text that it shows, which Vision reads from a
// screenshot, since System Events sees no buttons in System Settings.
//
//   ui click [--double] [--contains] TEXT [SECONDS]
//   ui wait [--contains] TEXT [SECONDS]
//   ui list
//   ui keys LETTERS        (virtual key codes, which password fields take)
//   ui key [--shift] [--command] return|tab|escape|a
//   ui scroll X Y LINES
import AppKit
import Vision

let source = CGEventSource(stateID: .hidSystemState)

struct Line {
  let text: String
  let recognized: VNRecognizedText
  let center: CGPoint
}

func lines() -> [Line] {
  let path = "/tmp/ui-screen.png"
  let capture = Process()
  capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
  capture.arguments = ["-x", path]
  try? capture.run()
  capture.waitUntilExit()
  guard let image = CGImageSourceCreateWithURL(URL(fileURLWithPath: path) as CFURL, nil)
    .flatMap({ CGImageSourceCreateImageAtIndex($0, 0, nil) })
  else { return [] }
  let request = VNRecognizeTextRequest()
  request.recognitionLevel = .accurate
  request.usesLanguageCorrection = false
  try? VNImageRequestHandler(cgImage: image).perform([request])
  let screen = CGDisplayBounds(CGMainDisplayID())
  return (request.results ?? []).compactMap { observation in
    guard let recognized = observation.topCandidates(1).first else { return nil }
    let box = observation.boundingBox
    return Line(
      text: recognized.string.trimmingCharacters(in: .whitespaces), recognized: recognized,
      center: CGPoint(x: screen.width * box.midX, y: screen.height * (1 - box.midY)))
  }
}

// Lowest match on the screen, where buttons sit, by the box of the text
// itself, since Vision reads buttons side by side as one line, such as
// "CancelAllow"
func find(_ text: String, contains: Bool, seconds: Double) -> CGPoint? {
  let deadline = Date().addingTimeInterval(seconds)
  repeat {
    let seen = lines()
    let screen = CGDisplayBounds(CGMainDisplayID())
    if let box = seen
      .filter({ contains ? $0.text.contains(text) : $0.text == text })
      .compactMap({ line in
        line.recognized.string.range(of: text).flatMap { try? line.recognized.boundingBox(for: $0)?.boundingBox }
      })
      .min(by: { $0.minY < $1.minY })
    {
      return CGPoint(x: screen.width * box.midX, y: screen.height * (1 - box.midY))
    }
    if Date() >= deadline {
      FileHandle.standardError.write(
        "ui: no \"\(text)\" among: \(seen.map(\.text).joined(separator: " | "))\n".data(using: .utf8)!)
      return nil
    }
    usleep(500_000)
  } while true
}

func click(_ point: CGPoint, count: Int) {
  CGEvent(mouseEventSource: source, mouseType: .mouseMoved, mouseCursorPosition: point, mouseButton: .left)?
    .post(tap: .cghidEventTap)
  usleep(200_000)
  for n in 1...count {
    for type in [CGEventType.leftMouseDown, .leftMouseUp] {
      let event = CGEvent(mouseEventSource: source, mouseType: type, mouseCursorPosition: point, mouseButton: .left)
      event?.setIntegerValueField(.mouseEventClickState, value: Int64(n))
      event?.post(tap: .cghidEventTap)
      usleep(40_000)
    }
    usleep(60_000)
  }
}

func press(_ code: CGKeyCode, flags: CGEventFlags = []) {
  for down in [true, false] {
    let event = CGEvent(keyboardEventSource: source, virtualKey: code, keyDown: down)
    event?.flags = flags
    event?.post(tap: .cghidEventTap)
    usleep(40_000)
  }
}

// Key codes of the US layout
let letters: [Character: CGKeyCode] = [
  "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9,
  "b": 11, "q": 12, "w": 13, "e": 14, "r": 15, "y": 16, "t": 17, "o": 31, "u": 32,
  "i": 34, "p": 35, "l": 37, "j": 38, "k": 40, "n": 45, "m": 46,
]
let named: [String: CGKeyCode] = ["return": 36, "tab": 48, "escape": 53, "a": 0]

var arguments = Array(CommandLine.arguments.dropFirst())
let command = arguments.isEmpty ? "" : arguments.removeFirst()
let double = arguments.contains("--double")
let contains = arguments.contains("--contains")
let flags = CGEventFlags()
  .union(arguments.contains("--shift") ? .maskShift : [])
  .union(arguments.contains("--command") ? .maskCommand : [])
arguments.removeAll { $0.hasPrefix("--") }

switch command {
case "click", "wait":
  let seconds = arguments.count > 1 ? Double(arguments[1]) ?? 10 : 10
  guard let point = find(arguments[0], contains: contains, seconds: seconds) else { exit(1) }
  if command == "click" { click(point, count: double ? 2 : 1) }
case "list":
  lines().forEach { print("\($0.text)\t\(Int($0.center.x))\t\(Int($0.center.y))") }
case "keys":
  arguments[0].forEach { letters[$0].map { press($0) } }
case "key":
  guard let code = named[arguments[0]] else { exit(2) }
  press(code, flags: flags)
case "scroll":
  let point = CGPoint(x: Double(arguments[0])!, y: Double(arguments[1])!)
  CGEvent(mouseEventSource: source, mouseType: .mouseMoved, mouseCursorPosition: point, mouseButton: .left)?
    .post(tap: .cghidEventTap)
  usleep(300_000)
  (0..<abs(Int(arguments[2])!)).forEach { _ in
    CGEvent(
      scrollWheelEvent2Source: source, units: .line, wheelCount: 1,
      wheel1: Int32(Int(arguments[2])! < 0 ? -10 : 10), wheel2: 0, wheel3: 0)?
      .post(tap: .cghidEventTap)
    usleep(50_000)
  }
default:
  FileHandle.standardError.write("ui: unknown command \(command)\n".data(using: .utf8)!)
  exit(2)
}
