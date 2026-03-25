import SwiftUI

enum HaneulchiTypography {
    // MARK: - Display / Page Title

    // Inter/SF Pro, 28–34pt, tight tracking (-0.03em to -0.04em)
    // Usage: authoritative route titles
    static let display = Font.custom("Inter", size: 32, relativeTo: .largeTitle)

    // MARK: - Section Heading

    // Inter/SF Pro, 18–20pt, moderate tight tracking
    // Usage: card/section headings
    static let sectionHeading = Font.custom("Inter", size: 18, relativeTo: .title3)

    // MARK: - Body

    // Inter/SF Pro, 14pt
    // Usage: readable operational text
    static let body = Font.custom("Inter", size: 14, relativeTo: .body)

    // MARK: - Body Small

    /// Inter/SF Pro, 13pt
    static let bodySmall = Font.custom("Inter", size: 13, relativeTo: .body)

    // MARK: - System Label

    // Space Grotesk, 12pt, wide tracking (0.12em–0.16em uppercase)
    // Usage: route tabs, badges, metadata chips
    static let systemLabel = Font.custom("Space Grotesk", size: 12, relativeTo: .caption)

    // MARK: - Compact Meta

    // Space Grotesk, 10–11pt, moderate tracking
    // Usage: IDs, coordinates, timestamps
    static let compactMeta = Font.custom("Space Grotesk", size: 10, relativeTo: .caption2)

    // MARK: - Legacy / Backwards Compat

    /// Keep these so existing call sites don't break
    static func heading(_ size: CGFloat) -> Font {
        Font.custom("Inter", size: size, relativeTo: .title3)
    }

    static func label(_ size: CGFloat) -> Font {
        Font.custom("Space Grotesk", size: size, relativeTo: .caption)
    }

    static let caption = Font.custom("Inter", size: 12, relativeTo: .caption)

    // MARK: - Tracking Constants

    /// Use these with .tracking() modifier
    enum Tracking {
        static let displayTight: CGFloat = -1.2 // approx -0.04em at 30pt
        static let headingSnug: CGFloat = -0.4 // approx -0.02em at 20pt
        static let labelWide: CGFloat = 1.4 // approx 0.12em at 12pt
        static let metaModerate: CGFloat = 0.9 // approx 0.08em at 11pt
        static let neutral: CGFloat = 0
    }

    // MARK: - Terminal Note

    // Terminal runtime font is NOT set here.
    // It is controlled via TerminalSettings (user-configured).
    // Do not apply chrome typography to terminal glyph rendering.
}
