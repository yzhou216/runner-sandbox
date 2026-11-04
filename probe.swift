// Posts scroll events and reports how they look after every session tap.
import CoreGraphics
import Foundation

let wheelTag: Int64 = 1
let gestureTag: Int64 = 2
var seen: [Int64: (line: Int64, point: Int64, horizontal: Int64)] = [:]

let callback: CGEventTapCallBack = { _, type, event, _ in
    if type == .scrollWheel {
        let tag = event.getIntegerValueField(.eventSourceUserData)
        let line = event.getIntegerValueField(.scrollWheelEventDeltaAxis1)
        let point = event.getIntegerValueField(.scrollWheelEventPointDeltaAxis1)
        let horizontal = event.getIntegerValueField(.scrollWheelEventDeltaAxis2)
        print("observed tag=\(tag) line=\(line) point=\(point) horizontal=\(horizontal)")
        if tag != 0 { seen[tag] = (line, point, horizontal) }
    }
    return Unmanaged.passUnretained(event)
}

guard let tap = CGEvent.tapCreate(
    tap: .cgAnnotatedSessionEventTap, place: .tailAppendEventTap, options: .listenOnly,
    eventsOfInterest: CGEventMask(1 << CGEventType.scrollWheel.rawValue),
    callback: callback, userInfo: nil)
else { print("cannot create the listening tap"); exit(2) }
CFRunLoopAddSource(CFRunLoopGetCurrent(),
    CFMachPortCreateRunLoopSource(nil, tap, 0), .commonModes)

let source = CGEventSource(stateID: .hidSystemState)
func post(tag: Int64, units: CGScrollEventUnit, phase: Int64) {
    let event = CGEvent(scrollWheelEvent2Source: source, units: units,
                        wheelCount: 2, wheel1: 3, wheel2: 2, wheel3: 0)!
    event.setIntegerValueField(.eventSourceUserData, value: tag)
    event.setIntegerValueField(.scrollWheelEventScrollPhase, value: phase)
    print("posting tag=\(tag) line=\(event.getIntegerValueField(.scrollWheelEventDeltaAxis1)) point=\(event.getIntegerValueField(.scrollWheelEventPointDeltaAxis1)) horizontal=\(event.getIntegerValueField(.scrollWheelEventDeltaAxis2))")
    event.post(tap: .cghidEventTap)
}

DispatchQueue.main.asyncAfter(deadline: .now() + 1) {
    post(tag: wheelTag, units: .line, phase: 0)
    post(tag: gestureTag, units: .pixel, phase: 1)
}
DispatchQueue.main.asyncAfter(deadline: .now() + 4) {
    guard let wheel = seen[wheelTag], let gesture = seen[gestureTag] else {
        print("FAIL: events not observed: \(seen)"); exit(1)
    }
    let wheelReversed = wheel.line < 0 && wheel.point < 0 && wheel.horizontal < 0
    let gestureKept = gesture.point > 0
    print("wheel reversed: \(wheelReversed), gesture untouched: \(gestureKept)")
    exit(wheelReversed && gestureKept ? 0 : 1)
}
CFRunLoopRun()
