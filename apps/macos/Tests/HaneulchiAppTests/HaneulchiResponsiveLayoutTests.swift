@testable import HaneulchiAppUI
import Testing

@Test("shared viewport classes classify the documented breakpoint thresholds")
func viewportClassifiesDocumentedThresholds() {
    #expect(HaneulchiViewportClass.forWidth(0) == .compact)
    #expect(HaneulchiViewportClass.forWidth(959) == .compact)
    #expect(HaneulchiViewportClass.forWidth(960) == .medium)
    #expect(HaneulchiViewportClass.forWidth(1239) == .medium)
    #expect(HaneulchiViewportClass.forWidth(1240) == .wide)
    #expect(HaneulchiViewportClass.forWidth(1519) == .wide)
    #expect(HaneulchiViewportClass.forWidth(1520) == .expanded)
}

@Test("shared route policy follows the viewport class without route-local thresholds")
func routeLayoutPolicyFollowsViewportClass() {
    let compact = HaneulchiViewportContext(width: 959).routeLayoutPolicy
    let medium = HaneulchiViewportContext(width: 960).routeLayoutPolicy
    let wide = HaneulchiViewportContext(width: 1240).routeLayoutPolicy
    let expanded = HaneulchiViewportContext(width: 1520).routeLayoutPolicy

    #expect(compact.showsExplorerColumn == false)
    #expect(compact.showsSupportingColumn == false)
    #expect(compact.stacksSupportingPanels == true)

    #expect(medium.showsExplorerColumn == false)
    #expect(medium.showsSupportingColumn == false)
    #expect(medium.stacksSupportingPanels == true)

    #expect(wide.showsExplorerColumn == false)
    #expect(wide.showsSupportingColumn == true)
    #expect(wide.stacksSupportingPanels == false)

    #expect(expanded.showsExplorerColumn == true)
    #expect(expanded.showsSupportingColumn == true)
    #expect(expanded.stacksSupportingPanels == false)
}

@Test("shared modal policy derives from shared modal tokens and clamps to them")
func modalWidthPolicyUsesSharedTokensAndClamps() {
    let compact = HaneulchiViewportContext(width: 0).modalWidthPolicy
    let wide = HaneulchiViewportContext(width: 1240).modalWidthPolicy
    let compactTokens = HaneulchiMetrics.Modal.compact
    let wideTokens = HaneulchiMetrics.Modal.wide

    #expect(compact.minimumWidth == compactTokens.minimumWidth)
    #expect(compact.idealWidth == compactTokens.idealWidth)
    #expect(compact.maximumWidth == compactTokens.maximumWidth)
    #expect(compact.clampedWidth(compactTokens.minimumWidth - 1) == compactTokens.minimumWidth)
    #expect(compact.clampedWidth(compactTokens.idealWidth) == compactTokens.idealWidth)
    #expect(compact.clampedWidth(compactTokens.maximumWidth + 1) == compactTokens.maximumWidth)

    #expect(wide.minimumWidth == wideTokens.minimumWidth)
    #expect(wide.idealWidth == wideTokens.idealWidth)
    #expect(wide.maximumWidth == wideTokens.maximumWidth)
    #expect(wide.clampedWidth(wideTokens.minimumWidth - 1) == wideTokens.minimumWidth)
    #expect(wide.clampedWidth(wideTokens.idealWidth) == wideTokens.idealWidth)
    #expect(wide.clampedWidth(wideTokens.maximumWidth + 1) == wideTokens.maximumWidth)
}
