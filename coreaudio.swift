// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// Sets the nominal sample rate of a Core Audio device, as Dante Controller
// sets that of the Dante Virtual Soundcard, or makes a device the default
// output of the system, as System Settings does.
//
// Usage: swift coreaudio.swift rate NAME HZ, or swift coreaudio.swift
// default NAME, with NAME the name of the device

import CoreAudio
import Foundation

func address(_ selector: AudioObjectPropertySelector) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress(
        mSelector: selector, mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain)
}

func devices() -> [AudioDeviceID] {
    var where_ = address(kAudioHardwarePropertyDevices)
    var size: UInt32 = 0
    AudioObjectGetPropertyDataSize(AudioObjectID(kAudioObjectSystemObject), &where_, 0, nil, &size)
    var ids = [AudioDeviceID](repeating: 0, count: Int(size) / MemoryLayout<AudioDeviceID>.size)
    AudioObjectGetPropertyData(AudioObjectID(kAudioObjectSystemObject), &where_, 0, nil, &size, &ids)
    return ids
}

func name(_ id: AudioDeviceID) -> String {
    var where_ = address(kAudioObjectPropertyName)
    var name: Unmanaged<CFString>?
    var size = UInt32(MemoryLayout<Unmanaged<CFString>?>.size)
    AudioObjectGetPropertyData(id, &where_, 0, nil, &size, &name)
    return (name?.takeRetainedValue() as String?) ?? ""
}

let arguments = CommandLine.arguments
guard arguments.count >= 3, let device = devices().first(where: { name($0) == arguments[2] }) else {
    print("no device \(arguments.dropFirst(2).first ?? "")")
    exit(1)
}
var status: OSStatus
switch arguments[1] {
case "rate":
    var rate = Float64(arguments[3]) ?? 48000
    var where_ = address(kAudioDevicePropertyNominalSampleRate)
    status = AudioObjectSetPropertyData(device, &where_, 0, nil, UInt32(MemoryLayout<Float64>.size), &rate)
case "default":
    var id = device
    var where_ = address(kAudioHardwarePropertyDefaultOutputDevice)
    status = AudioObjectSetPropertyData(
        AudioObjectID(kAudioObjectSystemObject), &where_, 0, nil, UInt32(MemoryLayout<AudioDeviceID>.size), &id)
default:
    print("unknown command \(arguments[1])")
    exit(2)
}
print("\(arguments[1]) \(arguments[2]): \(status)")
exit(status == noErr ? 0 : 1)
