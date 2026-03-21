import SwiftUI

enum HaneulchiTypography {
    static func heading(_ size: CGFloat) -> Font {
        Font.custom("Inter", size: size, relativeTo: .title3)
    }

    static func label(_ size: CGFloat) -> Font {
        Font.custom("Space Grotesk", size: size, relativeTo: .caption)
    }

    static let body = Font.custom("Inter", size: 14, relativeTo: .body)
    static let caption = Font.custom("Inter", size: 12, relativeTo: .caption)
}
