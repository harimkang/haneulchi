import CoreGraphics
import Foundation

enum InspectorSectionControlStyle: Equatable, Sendable {
    case segmented
    case compactScroll
}

enum ProjectFocusSessionContextStyle: Equatable, Sendable {
    case hidden
    case compactAffordance
    case column
}

enum ProjectFocusSupportingPanelLayoutStyle: Equatable, Sendable {
    case compact
    case regular
}

struct ProjectFocusWorkspaceLayoutMetrics: Equatable, Sendable {
    let outerPadding: CGFloat
    let columnSpacing: CGFloat
    let supportingColumnSpacing: CGFloat
    let sessionContextStyle: ProjectFocusSessionContextStyle
    let sessionColumnWidth: CGFloat
    let explorerColumnWidth: CGFloat
    let supportingColumnWidth: CGFloat
    let showsExplorerColumn: Bool
    let showsSupportingColumn: Bool
    let stacksSupportingPanelsInSharedColumn: Bool
    let inspectorControlStyle: InspectorSectionControlStyle
    let supportingPanelLayoutStyle: ProjectFocusSupportingPanelLayoutStyle

    var showsSessionColumn: Bool {
        sessionContextStyle == .column
    }

    var showsCompactSessionAffordance: Bool {
        sessionContextStyle == .compactAffordance
    }

    static func forPreset(
        _ preset: ProjectFocusLayoutPreset,
        viewportContext: HaneulchiViewportContext = .init(
            width: HaneulchiMetrics.Responsive.expandedWidth,
        ),
        inspectorSectionCount: Int = InspectorSection.allCases.count,
    ) -> Self {
        switch preset {
        case .fullTerminal:
            return Self(
                outerPadding: HaneulchiMetrics.Workspace.outerPadding,
                columnSpacing: HaneulchiMetrics.Workspace.columnGap,
                supportingColumnSpacing: HaneulchiMetrics.Workspace.supportingSectionGap,
                sessionContextStyle: .hidden,
                sessionColumnWidth: 0,
                explorerColumnWidth: 0,
                supportingColumnWidth: 0,
                showsExplorerColumn: false,
                showsSupportingColumn: false,
                stacksSupportingPanelsInSharedColumn: false,
                inspectorControlStyle: .segmented,
                supportingPanelLayoutStyle: .regular,
            )
        case .explorerTerminalInspector:
            let routePolicy = viewportContext.routeLayoutPolicy
            let sessionContextStyle: ProjectFocusSessionContextStyle = if routePolicy
                .showsCompactSessionContext
            {
                .compactAffordance
            } else if routePolicy.showsSessionColumn {
                .column
            } else {
                .hidden
            }
            let supportingPanelLayoutStyle: ProjectFocusSupportingPanelLayoutStyle =
                viewportContext.viewportClass == .expanded
                    ? .regular
                    : .compact
            let controlStyle: InspectorSectionControlStyle =
                supportingPanelLayoutStyle == .compact
                    || inspectorSectionCount > HaneulchiMetrics.Workspace
                    .inspectorCompactSectionLimit
                    ? .compactScroll
                    : .segmented

            return Self(
                outerPadding: HaneulchiMetrics.Workspace.outerPadding,
                columnSpacing: HaneulchiMetrics.Workspace.columnGap,
                supportingColumnSpacing: HaneulchiMetrics.Workspace.supportingSectionGap,
                sessionContextStyle: sessionContextStyle,
                sessionColumnWidth: routePolicy.showsSessionColumn
                    ? HaneulchiMetrics.Panel.sessionStackWidth
                    : 0,
                explorerColumnWidth: routePolicy.showsExplorerColumn
                    ? HaneulchiMetrics.Panel.explorerColumnWidth
                    : 0,
                supportingColumnWidth: routePolicy.showsSupportingColumn
                    ? HaneulchiMetrics.Panel.supportingColumnWidth
                    : 0,
                showsExplorerColumn: routePolicy.showsExplorerColumn,
                showsSupportingColumn: routePolicy.showsSupportingColumn,
                stacksSupportingPanelsInSharedColumn: routePolicy.showsSupportingColumn,
                inspectorControlStyle: controlStyle,
                supportingPanelLayoutStyle: supportingPanelLayoutStyle,
            )
        }
    }
}
