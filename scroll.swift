// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// Scrolls whatever lies under the point (620, 400) to its end
import CoreGraphics
import Foundation

let point = CGPoint(x: 620, y: 400)
CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: point, mouseButton: .left)?
  .post(tap: .cghidEventTap)
usleep(300_000)
for _ in 0..<40 {
  CGEvent(scrollWheelEvent2Source: nil, units: .line, wheelCount: 1, wheel1: -10, wheel2: 0, wheel3: 0)?
    .post(tap: .cghidEventTap)
  usleep(50_000)
}
