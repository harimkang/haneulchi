@testable import HaneulchiAppUI
import Testing

@Test(
    "review queue stacks master-detail in compact and medium widths, then splits from wide upward",
)
func reviewQueuePresentationModesFollowViewportClasses() {
    let compact = ReviewQueueResponsiveLayout(viewportClass: .compact)
    let medium = ReviewQueueResponsiveLayout(viewportClass: .medium)
    let wide = ReviewQueueResponsiveLayout(viewportClass: .wide)
    let expanded = ReviewQueueResponsiveLayout(viewportClass: .expanded)

    #expect(compact.mode == .stacked)
    #expect(compact.showsFixedMasterColumn == false)
    #expect(compact.stacksDecisionPanel == true)

    #expect(medium.mode == .stacked)
    #expect(medium.showsFixedMasterColumn == false)
    #expect(medium.stacksDecisionPanel == true)

    #expect(wide.mode == .split)
    #expect(wide.showsFixedMasterColumn == true)
    #expect(wide.stacksDecisionPanel == false)

    #expect(expanded.mode == .split)
    #expect(expanded.showsFixedMasterColumn == true)
    #expect(expanded.stacksDecisionPanel == false)
}

@Test(
    "review evidence pack collapses facts, file paths, and metric tiles for compact and medium widths",
)
func reviewEvidencePackLayoutAvoidsFixedWidthOverflowOnNarrowRoutes() {
    let compact = ReviewEvidencePackResponsiveLayout(viewportClass: .compact)
    let medium = ReviewEvidencePackResponsiveLayout(viewportClass: .medium)
    let wide = ReviewEvidencePackResponsiveLayout(viewportClass: .wide)

    #expect(compact.factRowStyle == .stacked)
    #expect(compact.usesFixedFactLabelColumn == false)
    #expect(compact.metricTileColumnCount == 1)
    #expect(compact.allowsWrappedTouchedFiles == true)

    #expect(medium.factRowStyle == .stacked)
    #expect(medium.usesFixedFactLabelColumn == false)
    #expect(medium.metricTileColumnCount == 2)
    #expect(medium.allowsWrappedTouchedFiles == true)

    #expect(wide.factRowStyle == .inline)
    #expect(wide.usesFixedFactLabelColumn == true)
    #expect(wide.metricTileColumnCount == 3)
    #expect(wide.allowsWrappedTouchedFiles == false)
}

@Test("review queue adds vertical overflow access when master-detail stacks")
func reviewQueueAddsVerticalOverflowAccessOnStackedLayouts() {
    let compact = ReviewQueueResponsiveLayout(viewportClass: .compact)
    let medium = ReviewQueueResponsiveLayout(viewportClass: .medium)
    let wide = ReviewQueueResponsiveLayout(viewportClass: .wide)

    #expect(compact.requiresVerticalOverflowScroll == true)
    #expect(medium.requiresVerticalOverflowScroll == true)
    #expect(wide.requiresVerticalOverflowScroll == false)
}

@Test("review queue split mode uses independent pane scrolling for master and detail reachability")
func reviewQueueSplitModeUsesIndependentPaneScrolling() {
    let wide = ReviewQueueResponsiveLayout(viewportClass: .wide)
    let expanded = ReviewQueueResponsiveLayout(viewportClass: .expanded)

    #expect(wide.usesIndependentPaneScrolling == true)
    #expect(expanded.usesIndependentPaneScrolling == true)
    #expect(wide.showsFixedMasterColumn == true)
}
