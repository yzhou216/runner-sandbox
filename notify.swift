// Posts the notification System Settings sends when Accessibility access changes.
import Foundation
DistributedNotificationCenter.default().postNotificationName(
    NSNotification.Name("com.apple.accessibility.api"), object: nil, userInfo: nil,
    deliverImmediately: true)
