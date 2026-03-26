@testable import HaneulchiApp
import SwiftUI
import Testing

@Test("signal accents preserve semantic priorities")
func signalAccentMappingPreservesSemanticPriorities() {
    #expect(HaneulchiSignalAccent.from(.reviewReady) == .reviewReady)
    #expect(HaneulchiSignalAccent.from(.waitingInput) == .warning)
    #expect(HaneulchiSignalAccent.from(.blocked) == .error)
    #expect(HaneulchiSignalAccent.from(.manualTakeover) == .manual)
}

@Test("ops strip metrics keep stable ordering")
func monolithMetricOrderingStaysStable() {
    let metrics = HaneulchiMonolithMetric.defaultOrder(
        cadence: "15000ms",
        lastTick: "tick",
        nextTick: "scheduled",
        reconcile: "none",
        slots: "2/4",
        workflow: "workflow_ok",
        tracker: "tracker_bound",
        queue: "0 retry · 0 claimed",
        paused: "no",
    )

    #expect(metrics.map(\.label) == [
        "cadence",
        "last tick",
        "next tick",
        "reconcile",
        "slots",
        "workflow",
        "tracker",
        "queue",
        "paused",
    ])
}

@Test("surface grammar views can be constructed with minimal data")
@MainActor
func surfaceGrammarViewsCanBeConstructed() {
    let metric = HaneulchiMonolithMetric(
        id: "workflow",
        label: "workflow",
        value: "workflow_ok",
        accent: .success,
    )

    let deck = HaneulchiHeaderDeck(title: "Control Tower", subtitle: "Operator overview") {
        EmptyView()
    }
    let strip = HaneulchiMonolithStrip(metrics: [metric]) {
        EmptyView()
    }
    let panel = HaneulchiOpsRailPanel(title: "Attention", count: 1) {
        EmptyView()
    }
    let row = HaneulchiSignalRow(
        accent: .warning,
        eyebrow: "UNREAD",
        title: "Needs input",
        summary: "Operator answer required.",
        meta: "project/demo",
    ) {
        EmptyView()
    }

    #expect(String(describing: type(of: deck)).contains("HaneulchiHeaderDeck"))
    #expect(String(describing: type(of: strip)).contains("HaneulchiMonolithStrip"))
    #expect(String(describing: type(of: panel)).contains("HaneulchiOpsRailPanel"))
    #expect(String(describing: type(of: row)).contains("HaneulchiSignalRow"))
}

@Test("metric tiles accept monolith metric models")
@MainActor
func metricTileUsesMonolithMetricModel() {
    let metric = HaneulchiMonolithMetric(
        id: "workflow",
        label: "workflow",
        value: "workflow_ok",
        accent: .success,
    )

    let tile = HaneulchiMetricTile(metric: metric)

    #expect(String(describing: type(of: tile)).contains("HaneulchiMetricTile"))
}

@Test(
    "operational screens share screen padding, section spacing, column gap, and supporting rail width",
)
func operationalScreenLayoutMetricsUseSharedRhythm() {
    let layout = HaneulchiOperationalLayoutMetrics.standard

    #expect(layout.screenPadding == HaneulchiMetrics.Padding.pageCompact)
    #expect(layout.sectionSpacing == HaneulchiMetrics.Spacing.lg)
    #expect(layout.columnSpacing == HaneulchiMetrics.Workspace.columnGap)
    #expect(layout.gridSpacing == HaneulchiMetrics.Workspace.columnGap)
    #expect(layout.supportingRailWidth == HaneulchiMetrics.Panel.supportingColumnWidth)
    #expect(layout.decisionRailWidth == 216)
}

@Test(
    "review surfaces reuse the shared supporting rail and keep the decision rail narrower than detail content",
)
func reviewScreenLayoutKeepsDecisionRailSecondary() {
    let layout = HaneulchiOperationalLayoutMetrics.standard

    #expect(layout.decisionRailWidth < layout.supportingRailWidth)
}

@Test("operational screen headers align to the same horizontal baseline as their body content")
func operationalScreenHeaderUsesSharedBaseline() {
    let layout = HaneulchiOperationalLayoutMetrics.standard

    #expect(layout.headerInnerPadding == 0)
    #expect(layout.screenPadding == HaneulchiMetrics.Padding.pageCompact)
}
