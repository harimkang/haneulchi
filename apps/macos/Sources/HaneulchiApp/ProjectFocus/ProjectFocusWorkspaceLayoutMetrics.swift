import CoreGraphics
import Foundation

enum InspectorSectionControlStyle: Equatable, Sendable {
    case segmented
    case compactScroll
}

struct ProjectFocusWorkspaceLayoutMetrics: Equatable, Sendable {
    let outerPadding: CGFloat
    let columnSpacing: CGFloat
    let supportingColumnSpacing: CGFloat
    let sessionColumnWidth: CGFloat
    let explorerColumnWidth: CGFloat
    let supportingColumnWidth: CGFloat
    let stacksSupportingPanelsInSharedColumn: Bool
    let inspectorControlStyle: InspectorSectionControlStyle

    static func forPreset(
        _ preset: ProjectFocusLayoutPreset,
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
                stacksSupportingPanelsInSharedColumn: false,
                inspectorControlStyle: .segmented,
            )
        case .explorerTerminalInspector:
            let controlStyle: InspectorSectionControlStyle =
                inspectorSectionCount > HaneulchiMetrics.Workspace.inspectorCompactSectionLimit
                    ? .compactScroll
                    : .segmented

            return Self(
                outerPadding: HaneulchiMetrics.Workspace.outerPadding,
                columnSpacing: HaneulchiMetrics.Workspace.columnGap,
                supportingColumnSpacing: HaneulchiMetrics.Workspace.supportingSectionGap,
                sessionColumnWidth: HaneulchiMetrics.Panel.sessionStackWidth,
                explorerColumnWidth: HaneulchiMetrics.Panel.explorerColumnWidth,
                supportingColumnWidth: HaneulchiMetrics.Panel.supportingColumnWidth,
                stacksSupportingPanelsInSharedColumn: true,
                inspectorControlStyle: controlStyle,
            )
        }
    }
}
