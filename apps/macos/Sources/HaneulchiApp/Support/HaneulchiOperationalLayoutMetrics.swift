import CoreGraphics
import Foundation

struct HaneulchiOperationalLayoutMetrics: Equatable, Sendable {
    let screenPadding: CGFloat
    let headerInnerPadding: CGFloat
    let sectionSpacing: CGFloat
    let columnSpacing: CGFloat
    let gridSpacing: CGFloat
    let supportingRailWidth: CGFloat
    let decisionRailWidth: CGFloat

    static let standard = Self(
        screenPadding: HaneulchiMetrics.Padding.pageCompact,
        headerInnerPadding: 0,
        sectionSpacing: HaneulchiMetrics.Spacing.lg,
        columnSpacing: HaneulchiMetrics.Workspace.columnGap,
        gridSpacing: HaneulchiMetrics.Workspace.columnGap,
        supportingRailWidth: HaneulchiMetrics.Panel.supportingColumnWidth,
        decisionRailWidth: 216,
    )
}
