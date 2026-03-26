import CoreGraphics
import Foundation

enum InspectorSectionControlStyle: Equatable, Sendable {
    case segmented
    case compactScroll
}

struct ProjectFocusWorkspaceLayoutMetrics: Equatable, Sendable {
    private static let supportingColumnCollapseWidth: CGFloat = 980
    private static let explorerColumnCollapseWidth: CGFloat = 1200

    let outerPadding: CGFloat
    let columnSpacing: CGFloat
    let supportingColumnSpacing: CGFloat
    let sessionColumnWidth: CGFloat
    let explorerColumnWidth: CGFloat
    let supportingColumnWidth: CGFloat
    let showsExplorerColumn: Bool
    let showsSupportingColumn: Bool
    let stacksSupportingPanelsInSharedColumn: Bool
    let inspectorControlStyle: InspectorSectionControlStyle

    static func forPreset(
        _ preset: ProjectFocusLayoutPreset,
        availableWidth: CGFloat = .greatestFiniteMagnitude,
        inspectorSectionCount: Int = InspectorSection.allCases.count,
    ) -> Self {
        switch preset {
        case .fullTerminal:
            return Self(
                outerPadding: HaneulchiMetrics.Workspace.outerPadding,
                columnSpacing: HaneulchiMetrics.Workspace.columnGap,
                supportingColumnSpacing: HaneulchiMetrics.Workspace.supportingSectionGap,
                sessionColumnWidth: 0,
                explorerColumnWidth: 0,
                supportingColumnWidth: 0,
                showsExplorerColumn: false,
                showsSupportingColumn: false,
                stacksSupportingPanelsInSharedColumn: false,
                inspectorControlStyle: .segmented,
            )
        case .explorerTerminalInspector:
            let showsSupportingColumn = availableWidth >= supportingColumnCollapseWidth
            let showsExplorerColumn = availableWidth >= explorerColumnCollapseWidth
            let controlStyle: InspectorSectionControlStyle =
                inspectorSectionCount > HaneulchiMetrics.Workspace.inspectorCompactSectionLimit
                    ? .compactScroll
                    : .segmented

            return Self(
                outerPadding: HaneulchiMetrics.Workspace.outerPadding,
                columnSpacing: HaneulchiMetrics.Workspace.columnGap,
                supportingColumnSpacing: HaneulchiMetrics.Workspace.supportingSectionGap,
                sessionColumnWidth: HaneulchiMetrics.Panel.sessionStackWidth,
                explorerColumnWidth: showsExplorerColumn
                    ? HaneulchiMetrics.Panel.explorerColumnWidth
                    : 0,
                supportingColumnWidth: showsSupportingColumn
                    ? HaneulchiMetrics.Panel.supportingColumnWidth
                    : 0,
                showsExplorerColumn: showsExplorerColumn,
                showsSupportingColumn: showsSupportingColumn,
                stacksSupportingPanelsInSharedColumn: showsSupportingColumn,
                inspectorControlStyle: controlStyle,
            )
        }
    }
}
