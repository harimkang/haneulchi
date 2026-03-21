import AppKit
import SwiftUI

enum HaneulchiChrome {
    enum Spacing {
        static let screenPadding: CGFloat = 24
        static let panelGap: CGFloat = 20
        static let panelPadding: CGFloat = 20
        static let itemGap: CGFloat = 12
    }

    enum Colors {
        static let appBackground = Color(nsColor: NSColor(
            calibratedRed: 0.10,
            green: 0.11,
            blue: 0.13,
            alpha: 1
        ))
        static let surfaceBase = Color(nsColor: NSColor(
            calibratedRed: 0.14,
            green: 0.15,
            blue: 0.18,
            alpha: 1
        ))
        static let surfaceRaised = Color(nsColor: NSColor(
            calibratedRed: 0.17,
            green: 0.18,
            blue: 0.22,
            alpha: 1
        ))
        static let surfaceMuted = Color(nsColor: NSColor(
            calibratedRed: 0.12,
            green: 0.13,
            blue: 0.16,
            alpha: 1
        ))
        static let primaryPanel = surfaceBase
        static let secondaryPanel = surfaceMuted
        static let mutedText = Color.secondary
        static let accent = Color(nsColor: .systemTeal)
        static let warning = Color(nsColor: .systemOrange)
        static let blocked = Color(nsColor: .systemRed)
        static let ready = Color(nsColor: .systemGreen)
        static let unread = Color(nsColor: .systemYellow)
    }
}
