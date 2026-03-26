import CoreGraphics
import SwiftUI

enum HaneulchiViewportClass: Int, CaseIterable, Comparable, Sendable {
    case compact
    case medium
    case wide
    case expanded

    static func < (lhs: Self, rhs: Self) -> Bool {
        lhs.rawValue < rhs.rawValue
    }

    static func forWidth(_ width: CGFloat) -> Self {
        switch width {
        case ..<HaneulchiMetrics.Responsive.mediumWidth:
            .compact
        case ..<HaneulchiMetrics.Responsive.wideWidth:
            .medium
        case ..<HaneulchiMetrics.Responsive.expandedWidth:
            .wide
        default:
            .expanded
        }
    }
}

enum HaneulchiShellChromeDensity: Equatable, Sendable {
    case compact
    case regular
}

enum HaneulchiSurfaceTrailingActionLayout: Equatable, Sendable {
    case inline
    case stacked
}

struct HaneulchiSurfaceLayoutPolicy: Equatable, Sendable {
    let viewportClass: HaneulchiViewportClass

    var trailingActionLayout: HaneulchiSurfaceTrailingActionLayout {
        viewportClass == .compact ? .stacked : .inline
    }

    var horizontalPadding: CGFloat {
        viewportClass == .compact ? HaneulchiMetrics.Padding.compact : HaneulchiMetrics.Padding.card
    }

    var buttonHorizontalPadding: CGFloat {
        viewportClass == .compact ? HaneulchiMetrics.Spacing.sm : HaneulchiMetrics.Spacing.md
    }
}

struct HaneulchiCompactTopBarChipPresentation: Equatable, Sendable {
    let visibleChips: [AppShellChromeState.Chip]
    let overflowChip: AppShellChromeState.Chip?
}

struct HaneulchiCompactBottomStripPresentation: Equatable, Sendable {
    let items: [AppShellChromeState.StripItem]
    let transientNotice: String?
}

struct HaneulchiViewportContext: Equatable, Sendable {
    let width: CGFloat
    let viewportClass: HaneulchiViewportClass

    init(width: CGFloat) {
        self.width = width
        viewportClass = .forWidth(width)
    }

    init(shellWidth: CGFloat) {
        self.init(width: Self.contentWidth(forShellWidth: shellWidth))
    }

    private static func contentWidth(forShellWidth shellWidth: CGFloat) -> CGFloat {
        HaneulchiMetrics.clamped(
            shellWidth - HaneulchiMetrics.Shell.railWidth,
            to: 0 ... .greatestFiniteMagnitude,
        )
    }

    var routeLayoutPolicy: HaneulchiRouteLayoutPolicy {
        .init(viewportClass: viewportClass)
    }

    var modalWidthPolicy: HaneulchiModalWidthPolicy {
        .init(viewportClass: viewportClass)
    }

    var surfaceLayoutPolicy: HaneulchiSurfaceLayoutPolicy {
        .init(viewportClass: viewportClass)
    }

    var shellChromeDensity: HaneulchiShellChromeDensity {
        viewportClass >= .wide ? .regular : .compact
    }

    func compactTopBarChipPresentation(
        for chips: [AppShellChromeState.Chip],
        visibleLimit: Int,
    ) -> HaneulchiCompactTopBarChipPresentation {
        let rankedChips = chips.enumerated()
            .sorted { lhs, rhs in
                let lhsPriority = Self.compactionPriority(for: lhs.element.tone)
                let rhsPriority = Self.compactionPriority(for: rhs.element.tone)

                if lhsPriority != rhsPriority {
                    return lhsPriority > rhsPriority
                }

                return lhs.offset < rhs.offset
            }
            .map(\.element)

        let visibleChipCount = max(0, visibleLimit)
        let visibleChips = Array(rankedChips.prefix(visibleChipCount))
        let hiddenChips = Array(rankedChips.dropFirst(visibleChips.count))

        let overflowChip: AppShellChromeState.Chip? = hiddenChips.isEmpty
            ? nil
            : .init(
                title: "+\(hiddenChips.count)",
                tone: Self.strongestTone(from: hiddenChips),
            )

        return .init(
            visibleChips: visibleChips,
            overflowChip: overflowChip,
        )
    }

    func compactBottomStripPresentation(
        items: [AppShellChromeState.StripItem],
        transientNotice: String?,
    ) -> HaneulchiCompactBottomStripPresentation {
        .init(
            items: items,
            transientNotice: transientNotice.map(Self.compactTransientNotice),
        )
    }

    private static func compactionPriority(for tone: WarningFlag?) -> Int {
        switch tone {
        case .failed:
            3
        case .degraded:
            2
        case .unread:
            1
        case nil:
            0
        }
    }

    private static func strongestTone(from chips: [AppShellChromeState.Chip]) -> WarningFlag? {
        chips.compactMap(\.tone).max { lhs, rhs in
            compactionPriority(for: lhs) < compactionPriority(for: rhs)
        }
    }

    private static func compactTransientNotice(_ notice: String) -> String {
        let limit = 28

        guard notice.count > limit else {
            return notice
        }

        return String(notice.prefix(limit - 1)) + "…"
    }
}

struct HaneulchiRouteLayoutPolicy: Equatable, Sendable {
    let viewportClass: HaneulchiViewportClass

    var showsSessionColumn: Bool {
        viewportClass >= .medium
    }

    var showsCompactSessionContext: Bool {
        viewportClass == .compact
    }

    var showsExplorerColumn: Bool {
        viewportClass == .expanded
    }

    var showsSupportingColumn: Bool {
        viewportClass >= .wide
    }

    var stacksSupportingPanels: Bool {
        viewportClass <= .medium
    }
}

struct HaneulchiModalWidthPolicy: Equatable, Sendable {
    private let tokens: HaneulchiMetrics.Modal.WidthTokens

    init(viewportClass: HaneulchiViewportClass) {
        tokens = HaneulchiMetrics.Modal.tokens(for: viewportClass)
    }

    var minimumWidth: CGFloat {
        tokens.minimumWidth
    }

    var idealWidth: CGFloat {
        tokens.idealWidth
    }

    var maximumWidth: CGFloat {
        tokens.maximumWidth
    }

    func clampedWidth(_ width: CGFloat) -> CGFloat {
        HaneulchiMetrics.clamped(width, to: minimumWidth ... maximumWidth)
    }
}

extension EnvironmentValues {
    @Entry var viewportContext: HaneulchiViewportContext = .init(width: 0)
}
