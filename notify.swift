// SPDX-FileCopyrightText: 2026 runner-sandbox contributors
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// Posts the notification System Settings sends when Accessibility access changes.
import Foundation
DistributedNotificationCenter.default().postNotificationName(
    NSNotification.Name("com.apple.accessibility.api"), object: nil, userInfo: nil,
    deliverImmediately: true)
