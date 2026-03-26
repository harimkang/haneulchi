import AppKit
import SwiftUI

enum HaneulchiTypography {
    enum Token {
        case display
        case text
        case label
        case mono
    }

    enum SystemFallback: Equatable {
        case standard
        case monospaced
    }

    enum Source: Equatable {
        case custom(String)
        case system(SystemFallback)
    }

    enum Family {
        static let display = "SF Pro Display"
        static let text = "SF Pro Text"
        static let label = "SF Compact Text"
        static let mono = "SF Mono"
    }

    // MARK: - Display / Page Title

    // Inter/SF Pro, 28–34pt, tight tracking (-0.03em to -0.04em)
    // Usage: authoritative route titles
    static let display = resolvedFont(for: .display, size: 32, relativeTo: .largeTitle)

    // MARK: - Section Heading

    // Inter/SF Pro, 18–20pt, moderate tight tracking
    // Usage: card/section headings
    static let sectionHeading = resolvedFont(
        for: .display,
        size: 18,
        relativeTo: .title3,
    )

    // MARK: - Body

    // Inter/SF Pro, 14pt
    // Usage: readable operational text
    static let body = resolvedFont(for: .text, size: 14, relativeTo: .body)

    // MARK: - Body Small

    /// Inter/SF Pro, 13pt
    static let bodySmall = resolvedFont(for: .text, size: 13, relativeTo: .body)

    // MARK: - Deck Subtitle

    /// Inter/SF Pro, 13pt
    static let deckSubtitle = resolvedFont(for: .text, size: 13, relativeTo: .subheadline)

    // MARK: - System Label

    // Space Grotesk, 12pt, wide tracking (0.12em–0.16em uppercase)
    // Usage: route tabs, badges, metadata chips
    static let systemLabel = resolvedFont(for: .label, size: 12, relativeTo: .caption)

    // MARK: - Ops Value

    /// Space Grotesk, 11pt, compact ops-strip values
    static let opsValue = resolvedFont(for: .label, size: 11, relativeTo: .caption)

    // MARK: - Compact Meta

    // Space Grotesk, 10–11pt, moderate tracking
    // Usage: IDs, coordinates, timestamps
    static let compactMeta = resolvedFont(for: .label, size: 10, relativeTo: .caption2)

    // MARK: - Legacy / Backwards Compat

    /// Keep these so existing call sites don't break
    static func heading(_ size: CGFloat) -> Font {
        resolvedFont(for: .display, size: size, relativeTo: .title3)
    }

    static func label(_ size: CGFloat) -> Font {
        resolvedFont(for: .label, size: size, relativeTo: .caption)
    }

    static let caption = resolvedFont(for: .text, size: 12, relativeTo: .caption)

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

    static func source(
        for token: Token,
        isAvailable: (String) -> Bool = { family in NSFont(name: family, size: 12) != nil },
    ) -> Source {
        for family in preferredFamilies(for: token) where isAvailable(family) {
            return .custom(family)
        }

        switch token {
        case .mono:
            return .system(.monospaced)
        case .display, .text, .label:
            return .system(.standard)
        }
    }

    private static func resolvedFont(
        for token: Token,
        size: CGFloat,
        relativeTo textStyle: Font.TextStyle,
    ) -> Font {
        switch source(for: token) {
        case let .custom(family):
            .custom(family, size: size, relativeTo: textStyle)
        case let .system(fallback):
            systemFont(
                for: token,
                fallback: fallback,
                size: size,
            )
        }
    }

    private static func preferredFamilies(for token: Token) -> [String] {
        switch token {
        case .display:
            [Family.display, Family.text]
        case .text:
            [Family.text, Family.display]
        case .label:
            [Family.label, Family.text]
        case .mono:
            [Family.mono]
        }
    }

    private static func systemFont(
        for token: Token,
        fallback: SystemFallback,
        size: CGFloat,
    ) -> Font {
        switch fallback {
        case .monospaced:
            .system(size: size, weight: systemWeight(for: token), design: .monospaced)
        case .standard:
            .system(size: size, weight: systemWeight(for: token), design: .default)
        }
    }

    private static func systemWeight(for token: Token) -> Font.Weight {
        switch token {
        case .display:
            .semibold
        case .text:
            .regular
        case .label:
            .medium
        case .mono:
            .regular
        }
    }
}
