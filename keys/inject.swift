// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// Posts keys and mouse buttons as the hardware does, through CoreGraphics.
//
//   inject key [ctrl+][alt+][shift+]NAME   (a letter, a digit, left, right, end, pagedown)
//   inject button NUMBER X Y               (other mouse button, NUMBER 3 or 4, at X Y)
import AppKit

let source = CGEventSource(stateID: .hidSystemState)
let codes: [String: CGKeyCode] = [
  "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9,
  "b": 11, "q": 12, "w": 13, "e": 14, "r": 15, "y": 16, "t": 17, "3": 20, "o": 31,
  "u": 32, "i": 34, "p": 35, "l": 37, "j": 38, "k": 40, "n": 45, "m": 46,
  "left": 123, "right": 124, "end": 119, "pagedown": 121, "home": 115,
]
let modifiers: [String: (CGKeyCode, CGEventFlags)] = [
  "ctrl": (59, .maskControl), "alt": (58, .maskAlternate), "shift": (56, .maskShift),
]

func post(_ code: CGKeyCode, down: Bool, flags: CGEventFlags) {
  let event = CGEvent(keyboardEventSource: source, virtualKey: code, keyDown: down)
  event?.flags = flags
  event?.post(tap: .cghidEventTap)
  usleep(50_000)
}

func key(_ spec: String) {
  let names = spec.lowercased().split(separator: "+").map(String.init)
  guard let name = names.last, let code = codes[name] else {
    FileHandle.standardError.write("inject: no key in \(spec)\n".data(using: .utf8)!)
    exit(1)
  }
  let held = names.dropLast().compactMap { modifiers[$0] }
  var flags: CGEventFlags = []
  // The modifier keys go down first, each with the flags of those before it
  for (modifier, flag) in held {
    flags.insert(flag)
    post(modifier, down: true, flags: flags)
  }
  post(code, down: true, flags: flags)
  post(code, down: false, flags: flags)
  for (modifier, flag) in held.reversed() {
    flags.remove(flag)
    post(modifier, down: false, flags: flags)
  }
}

func button(_ number: Int64, _ point: CGPoint) {
  CGWarpMouseCursorPosition(point)
  CGEvent(mouseEventSource: source, mouseType: .mouseMoved, mouseCursorPosition: point, mouseButton: .left)?
    .post(tap: .cghidEventTap)
  usleep(300_000)
  // CGMouseButton has no case beyond the center button, so the number goes
  // in as the field of the event
  for type in [CGEventType.otherMouseDown, .otherMouseUp] {
    let event = CGEvent(mouseEventSource: source, mouseType: type, mouseCursorPosition: point, mouseButton: .center)
    event?.setIntegerValueField(.mouseEventButtonNumber, value: number)
    event?.post(tap: .cghidEventTap)
    usleep(60_000)
  }
}

let args = CommandLine.arguments
switch args.count > 1 ? args[1] : "" {
case "key" where args.count == 3: key(args[2])
case "button" where args.count == 5:
  button(Int64(args[2]) ?? 3, CGPoint(x: Double(args[3]) ?? 0, y: Double(args[4]) ?? 0))
default:
  FileHandle.standardError.write("usage: inject key SPEC | inject button N X Y\n".data(using: .utf8)!)
  exit(2)
}
