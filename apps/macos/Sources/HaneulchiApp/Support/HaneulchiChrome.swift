import SwiftUI

enum HaneulchiChrome {
    // MARK: - Spacing (unchanged)

    enum Spacing {
        static let screenPadding: CGFloat = 24
        static let densePadding: CGFloat = 16
        static let panelGap: CGFloat = 20
        static let panelPadding: CGFloat = 20
        static let itemGap: CGFloat = 12
    }

    // MARK: - Surface Tokens

    enum Surface {
        /// #131315 — main workspace, window background, terminal backdrop
        static let foundation = Color(red: 19 / 255, green: 19 / 255, blue: 21 / 255)
        /// #0E0E10 — inactive tabs, bottom strip, deep gutters
        static let recess = Color(red: 14 / 255, green: 14 / 255, blue: 16 / 255)
        /// #1B1B1D — columns, rails, passive cards
        static let base = Color(red: 27 / 255, green: 27 / 255, blue: 29 / 255)
        /// #2A2A2C — active cards, focused inspector modules
        static let raised = Color(red: 42 / 255, green: 42 / 255, blue: 44 / 255)
        /// #353437 @ 0.84 opacity by default, overridable — command palette, popovers, modals
        static func floating(_ opacity: Double = 0.84) -> Color {
            Color(red: 53 / 255, green: 52 / 255, blue: 55 / 255).opacity(opacity)
        }

        /// Scrim backdrop for modal overlays (command palette, sheets)
        static let scrim = Color.black.opacity(0.32)
    }

    // MARK: - Text Tokens

    enum Label {
        /// #E4E2E4 — primary readable text
        static let primary = Color(red: 228 / 255, green: 226 / 255, blue: 228 / 255)
        /// #C0C6D6 — secondary metadata
        static let secondary = Color(red: 192 / 255, green: 198 / 255, blue: 214 / 255)
        /// #8B91A0 — hints, inactive labels
        static let muted = Color(red: 139 / 255, green: 145 / 255, blue: 160 / 255)
    }

    // MARK: - Stroke Tokens

    enum Stroke {
        /// #414754 @ 0.28 — focus fallback, subtle separation
        static let ghost = Color(red: 65 / 255, green: 71 / 255, blue: 84 / 255).opacity(0.28)
    }

    // MARK: - Gradient Tokens

    enum Gradient {
        /// #AAC7FF — CTA gradient start, active nav
        static let primaryStart = Color(red: 170 / 255, green: 199 / 255, blue: 255 / 255)
        /// #3E90FF — CTA gradient end
        static let primaryEnd = Color(red: 62 / 255, green: 144 / 255, blue: 255 / 255)
        /// Linear gradient from primaryStart to primaryEnd
        static let primaryLinear = LinearGradient(
            colors: [primaryStart, primaryEnd],
            startPoint: .leading,
            endPoint: .trailing,
        )
    }

    // MARK: - Semantic State Tokens

    enum State {
        /// #42E355 — healthy/running indicator text/icon
        static let success = Color(red: 66 / 255, green: 227 / 255, blue: 85 / 255)
        /// #04C339 — active agent chip fill
        static let successSolid = Color(red: 4 / 255, green: 195 / 255, blue: 57 / 255)
        /// #FFB868 — waiting/retry emphasis text
        static let warning = Color(red: 255 / 255, green: 184 / 255, blue: 104 / 255)
        /// #CE7F00 — waiting input chip fill
        static let warningSolid = Color(red: 206 / 255, green: 127 / 255, blue: 0 / 255)
        /// #FFB4AB — blocked/error text and icons
        static let error = Color(red: 255 / 255, green: 180 / 255, blue: 171 / 255)
        /// #93000A — destructive chip fill
        static let errorSolid = Color(red: 147 / 255, green: 0 / 255, blue: 10 / 255)
    }

    // MARK: - Backwards Compatibility Aliases

    enum Colors {
        static let appBackground: Color = Surface.foundation
        static let surfaceBase: Color = Surface.base
        static let surfaceRaised: Color = Surface.raised
        static let surfaceMuted: Color = Surface.recess
        static let primaryPanel: Color = Surface.base
        static let secondaryPanel: Color = Surface.recess
        static let mutedText: Color = Label.muted
        static let accent: Color = Gradient.primaryEnd
        static let warning: Color = State.warning
        static let blocked: Color = State.error
        static let ready: Color = State.success
        static let unread: Color = State.warning
    }
}
