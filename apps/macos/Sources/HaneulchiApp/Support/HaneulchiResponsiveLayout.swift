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

struct HaneulchiViewportContext: Equatable, Sendable {
    let width: CGFloat
    let viewportClass: HaneulchiViewportClass

    init(width: CGFloat) {
        self.width = width
        viewportClass = .forWidth(width)
    }

    var routeLayoutPolicy: HaneulchiRouteLayoutPolicy {
        .init(viewportClass: viewportClass)
    }

    var modalWidthPolicy: HaneulchiModalWidthPolicy {
        .init(viewportClass: viewportClass)
    }
}

struct HaneulchiRouteLayoutPolicy: Equatable, Sendable {
    let viewportClass: HaneulchiViewportClass

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
