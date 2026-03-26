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
    let viewportClass: HaneulchiViewportClass

    var minimumWidth: CGFloat {
        switch viewportClass {
        case .compact:
            520
        case .medium:
            560
        case .wide, .expanded:
            640
        }
    }

    var idealWidth: CGFloat {
        switch viewportClass {
        case .compact:
            600
        case .medium:
            680
        case .wide:
            720
        case .expanded:
            760
        }
    }

    var maximumWidth: CGFloat {
        switch viewportClass {
        case .compact, .medium:
            720
        case .wide, .expanded:
            760
        }
    }

    func clampedWidth(_ width: CGFloat) -> CGFloat {
        HaneulchiMetrics.clamped(width, to: minimumWidth ... maximumWidth)
    }
}

extension EnvironmentValues {
    @Entry var viewportContext: HaneulchiViewportContext = .init(width: 0)
}
