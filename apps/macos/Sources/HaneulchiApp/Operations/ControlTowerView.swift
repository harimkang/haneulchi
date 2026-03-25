import SwiftUI

struct ControlTowerView: View {
    let model: ControlTowerViewModel
    let onAction: (AppShellAction) -> Void

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                HStack(alignment: .firstTextBaseline) {
                    Text("Control Tower")
                        .font(HaneulchiTypography.display)
                        .foregroundStyle(HaneulchiChrome.Label.primary)
                    Spacer()
                    Button("Quick Dispatch") {
                        onAction(.presentQuickDispatch(.controlTower))
                    }
                    .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                }

                opsMetricStrip

                HaneulchiSectionHeader(title: "Projects")

                ControlTowerProjectCardGrid(cards: model.projectCards) { _ in
                    onAction(.selectRoute(.projectFocus))
                }

                HStack(alignment: .top, spacing: HaneulchiChrome.Spacing.panelGap) {
                    AttentionQueueSummaryView(items: model.attentionItems) { item in
                        if let sessionID = item.targetSessionID {
                            onAction(.jumpToSession(sessionID))
                        } else {
                            onAction(.selectRoute(item.targetRoute))
                        }
                    }

                    RecentArtifactsTableView(items: model.recentArtifacts) { item in
                        onAction(.selectRoute(item.targetRoute))
                    }
                }
            }
            .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)
            .padding(.vertical, HaneulchiChrome.Spacing.panelGap)
            .padding(.bottom, HaneulchiChrome.Spacing.panelGap)
        }
        .background(HaneulchiChrome.Surface.foundation)
    }

    private var opsMetricStrip: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: HaneulchiChrome.Spacing.itemGap) {
                    HaneulchiMetricTile(
                        label: "cadence",
                        value: model.opsModel.cadenceLabel
                    )
                    HaneulchiMetricTile(
                        label: "last tick",
                        value: model.opsModel.lastTickLabel
                    )
                    HaneulchiMetricTile(
                        label: "next tick",
                        value: model.opsModel.nextTickLabel
                    )
                    HaneulchiMetricTile(
                        label: "reconcile",
                        value: model.opsModel.lastReconcileLabel
                    )
                    HaneulchiMetricTile(
                        label: "slots",
                        value: model.opsModel.slotLabel
                    )
                    HaneulchiMetricTile(
                        label: "workflow",
                        value: model.opsModel.workflowHealth
                    )
                    HaneulchiMetricTile(
                        label: "tracker",
                        value: model.opsModel.trackerHealth
                    )
                    HaneulchiMetricTile(
                        label: "queue",
                        value: model.opsModel.queueLabel
                    )
                    HaneulchiMetricTile(
                        label: "paused",
                        value: model.opsModel.paused ? "yes" : "no"
                    )
                }
            }

            HStack(spacing: HaneulchiChrome.Spacing.itemGap) {
                Button("Refresh") { onAction(.refreshShellSnapshot) }
                    .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                Button("Reconcile") { onAction(.reconcileAutomation) }
                    .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                Button("Reload") { onAction(.reloadWorkflow) }
                    .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
            }
        }
    }
}
