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

    init(rootWidth: CGFloat) {
        self.init(width: rootWidth)
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

    func drawerWidthPolicy(for role: HaneulchiDrawerWidthRole) -> HaneulchiDrawerWidthPolicy {
        .init(role: role)
    }

    func commandPaletteWidth(availableWidth: CGFloat?) -> CGFloat {
        HaneulchiResponsiveWidthResolution.resolvedWidth(
            preferredWidth: HaneulchiMetrics.Panel.commandPaletteMax,
            minimumWidth: HaneulchiMetrics.Panel.commandPaletteMin,
            maximumWidth: HaneulchiMetrics.Panel.commandPaletteMax,
            availableWidth: availableWidth,
        )
    }

    func contextDrawerWidth(availableWidth: CGFloat?) -> CGFloat {
        let policy = drawerWidthPolicy(for: .context)
        return policy.resolvedWidth(
            preferredWidth: policy.maximumWidth,
            availableWidth: availableWidth,
        )
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

struct ControlTowerResponsiveLayout: Equatable, Sendable {
    let projectGridColumnCount: Int
    let stacksLowerStage: Bool

    init(viewportClass: HaneulchiViewportClass) {
        switch viewportClass {
        case .compact:
            projectGridColumnCount = 1
            stacksLowerStage = true
        case .medium:
            projectGridColumnCount = 2
            stacksLowerStage = true
        case .wide:
            projectGridColumnCount = 2
            stacksLowerStage = false
        case .expanded:
            projectGridColumnCount = 3
            stacksLowerStage = false
        }
    }

    var usesDenseProjectGrid: Bool {
        projectGridColumnCount >= 3
    }

    func projectGridLayout(
        availableWidth: CGFloat,
        spacing: CGFloat,
        minimumCardWidth: CGFloat = 220,
        maximumCardWidth: CGFloat = 280,
    ) -> ControlTowerProjectGridLayout {
        let maxColumns = max(1, projectGridColumnCount)
        let normalizedWidth = max(0, availableWidth)

        let resolvedColumnCount: Int = if normalizedWidth > 0 {
            max(
                1,
                min(
                    maxColumns,
                    Int((normalizedWidth + spacing) / (minimumCardWidth + spacing)),
                ),
            )
        } else {
            maxColumns
        }

        let proposedWidth: CGFloat = if normalizedWidth > 0 {
            (normalizedWidth - spacing * CGFloat(resolvedColumnCount - 1))
                / CGFloat(resolvedColumnCount)
        } else {
            maximumCardWidth
        }

        let cardWidth = min(
            maximumCardWidth,
            max(minimumCardWidth, floor(proposedWidth)),
        )

        return ControlTowerProjectGridLayout(
            columnCount: resolvedColumnCount,
            cardWidth: normalizedWidth > 0 ? min(normalizedWidth, cardWidth) : cardWidth,
        )
    }
}

struct ControlTowerProjectGridLayout: Equatable, Sendable {
    let columnCount: Int
    let cardWidth: CGFloat
}

struct WelcomeReadinessResponsiveLayout: Equatable, Sendable {
    let usesSplitLauncher: Bool

    init(viewportClass: HaneulchiViewportClass) {
        usesSplitLauncher = viewportClass >= .medium
    }
}

enum ReviewQueueSurfaceMode: Equatable, Sendable {
    case stacked
    case split
}

struct ReviewQueueResponsiveLayout: Equatable, Sendable {
    let mode: ReviewQueueSurfaceMode
    let stacksDecisionPanel: Bool
    let masterColumnWidth: CGFloat?

    init(viewportClass: HaneulchiViewportClass) {
        switch viewportClass {
        case .compact, .medium:
            mode = .stacked
            stacksDecisionPanel = true
            masterColumnWidth = nil
        case .wide, .expanded:
            mode = .split
            stacksDecisionPanel = false
            masterColumnWidth = HaneulchiMetrics.Panel.supportingColumnWidth
        }
    }

    var showsFixedMasterColumn: Bool {
        masterColumnWidth != nil
    }

    var requiresVerticalOverflowScroll: Bool {
        mode == .stacked
    }

    var usesIndependentPaneScrolling: Bool {
        mode == .split
    }
}

enum ReviewEvidencePackFactRowStyle: Equatable, Sendable {
    case stacked
    case inline
}

struct ReviewEvidencePackResponsiveLayout: Equatable, Sendable {
    let factRowStyle: ReviewEvidencePackFactRowStyle
    let metricTileColumnCount: Int
    let allowsWrappedTouchedFiles: Bool

    init(viewportClass: HaneulchiViewportClass) {
        switch viewportClass {
        case .compact:
            factRowStyle = .stacked
            metricTileColumnCount = 1
            allowsWrappedTouchedFiles = true
        case .medium:
            factRowStyle = .stacked
            metricTileColumnCount = 2
            allowsWrappedTouchedFiles = true
        case .wide, .expanded:
            factRowStyle = .inline
            metricTileColumnCount = 3
            allowsWrappedTouchedFiles = false
        }
    }

    var usesFixedFactLabelColumn: Bool {
        factRowStyle == .inline
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

    func resolvedWidth(
        preferredWidth: CGFloat? = nil,
        availableWidth: CGFloat?,
    ) -> CGFloat {
        HaneulchiResponsiveWidthResolution.resolvedWidth(
            preferredWidth: preferredWidth ?? idealWidth,
            minimumWidth: minimumWidth,
            maximumWidth: maximumWidth,
            availableWidth: availableWidth,
        )
    }
}

enum HaneulchiDrawerWidthRole: Equatable, Sendable {
    case notification
    case context
}

struct HaneulchiDrawerWidthPolicy: Equatable, Sendable {
    let minimumWidth: CGFloat
    let idealWidth: CGFloat
    let maximumWidth: CGFloat

    init(role: HaneulchiDrawerWidthRole) {
        switch role {
        case .notification:
            minimumWidth = HaneulchiMetrics.Panel.inspectorMin
            idealWidth = HaneulchiMetrics.Panel.inspectorMin
            maximumWidth = HaneulchiMetrics.Panel.inspectorMax
        case .context:
            minimumWidth = 360
            idealWidth = 420
            maximumWidth = 520
        }
    }

    func clampedWidth(_ width: CGFloat) -> CGFloat {
        HaneulchiMetrics.clamped(width, to: minimumWidth ... maximumWidth)
    }

    func resolvedWidth(
        preferredWidth: CGFloat? = nil,
        availableWidth: CGFloat?,
    ) -> CGFloat {
        HaneulchiResponsiveWidthResolution.resolvedWidth(
            preferredWidth: preferredWidth ?? idealWidth,
            minimumWidth: minimumWidth,
            maximumWidth: maximumWidth,
            availableWidth: availableWidth,
        )
    }
}

private enum HaneulchiResponsiveWidthResolution {
    static func resolvedWidth(
        preferredWidth: CGFloat,
        minimumWidth: CGFloat,
        maximumWidth: CGFloat,
        availableWidth: CGFloat?,
    ) -> CGFloat {
        let targetWidth = HaneulchiMetrics.clamped(
            preferredWidth,
            to: minimumWidth ... maximumWidth,
        )

        guard let availableWidth else {
            return targetWidth
        }

        return min(targetWidth, availableWidth)
    }
}

extension EnvironmentValues {
    @Entry var viewportContext: HaneulchiViewportContext = .init(width: 0)
}
