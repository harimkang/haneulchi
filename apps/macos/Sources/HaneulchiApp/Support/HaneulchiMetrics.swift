import SwiftUI

enum HaneulchiMetrics {
    // MARK: - Spacing Scale (4pt grid)

    enum Spacing {
        static let xxs: CGFloat = 4
        static let xs: CGFloat = 8
        static let sm: CGFloat = 12
        static let md: CGFloat = 16
        static let lg: CGFloat = 24
        static let xl: CGFloat = 32
        static let xxl: CGFloat = 48
    }

    // MARK: - Padding

    enum Padding {
        static let card: CGFloat = 16 // card internal padding
        static let compact: CGFloat = 8 // compact/dense areas
        static let page: CGFloat = 32 // desktop page margin
        static let pageCompact: CGFloat = 24 // compact page margin
        static let columnGap: CGFloat = 24 // gap between columns
    }

    // MARK: - Operations Surface Metrics

    enum Operations {
        static let headerDeckMinHeight: CGFloat = 72
        static let opsStripMinHeight: CGFloat = 52
        static let opsRailWidth: CGFloat = 360
        static let signalRowMinHeight: CGFloat = 68
    }

    // MARK: - Shell Dimensions

    enum Shell {
        static let railWidth: CGFloat = 48
        static let topBarHeight: CGFloat = 36
        static let bottomStripHeight: CGFloat = 22
    }

    // MARK: - Column / Panel Widths

    enum Panel {
        static let explorerMin: CGFloat = 240
        static let explorerMax: CGFloat = 280
        static let inspectorMin: CGFloat = 320
        static let inspectorMax: CGFloat = 360
        static let boardColumnMin: CGFloat = 280
        static let commandPaletteMin: CGFloat = 640
        static let commandPaletteMax: CGFloat = 760
    }

    // MARK: - Corner Radii

    enum Radius {
        static let small: CGFloat = 2
        static let medium: CGFloat = 6
        static let large: CGFloat = 12
        static let pill: CGFloat = 999 // for fully-rounded chips/badges
    }

    // MARK: - Interactive Target Heights

    enum Target {
        static let compact: CGFloat = 32 // compact interactive min height
        static let primary: CGFloat = 44 // primary interactive min height
        static let row: CGFloat = 36 // standard list row height
        static let compactRow: CGFloat = 28 // dense list row height
    }

    // MARK: - Icon Sizes

    enum Icon {
        static let small: CGFloat = 14
        static let standard: CGFloat = 16
        static let medium: CGFloat = 18
        static let large: CGFloat = 24
    }

    // MARK: - Motion Durations (seconds)

    enum Motion {
        static let hoverShift: Double = 0.16 // 140–180ms
        static let pressedSelection: Double = 0.14 // 120–160ms
        static let panelRaise: Double = 0.20 // 180–220ms
        static let overlayEnter: Double = 0.24 // 220–260ms
    }
}
