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
        static let appBackground = Color(nsColor: .windowBackgroundColor)
        static let primaryPanel = Color(nsColor: .controlBackgroundColor)
        static let secondaryPanel = Color(nsColor: .underPageBackgroundColor)
        static let mutedText = Color.secondary
        static let warning = Color.orange
        static let blocked = Color.red
        static let ready = Color.green
    }
}
