import SwiftUI

struct ControlTowerView: View {
    let model: ControlTowerViewModel
    let onAction: (AppShellAction) -> Void

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                HaneulchiHeaderDeck(
                    title: "Control Tower",
                    subtitle: "Scan control-plane health and multi-project activity without leaving the operator surface.",
                ) {
                    HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                        Button("Resync") {
                            onAction(.refreshShellSnapshot)
                        }
                        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))

                        Button("New Session") {
                            onAction(.presentNewSessionSheet)
                        }
                        .buttonStyle(HaneulchiButtonStyle(variant: .primary))
                    }
                }

                ControlTowerOpsStripView(
                    model: model.opsModel,
                    onRefresh: { onAction(.refreshShellSnapshot) },
                    onReconcile: { onAction(.reconcileAutomation) },
                    onReload: { onAction(.reloadWorkflow) },
                )

                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.md) {
                    HStack(alignment: .firstTextBaseline) {
                        Text("Project Matrix")
                            .font(HaneulchiTypography.sectionHeading)
                            .foregroundStyle(HaneulchiChrome.Label.primary)
                        Text("\(model.projectCards.count)")
                            .font(HaneulchiTypography.compactMeta)
                            .tracking(HaneulchiTypography.Tracking.metaModerate)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                        Spacer()
                        Button("Quick Dispatch") {
                            onAction(.presentQuickDispatch(.controlTower))
                        }
                        .buttonStyle(HaneulchiButtonStyle(variant: .tertiary))
                    }

                    ControlTowerProjectCardGrid(cards: model.projectCards) { _ in
                        onAction(.selectRoute(.projectFocus))
                    }

                    ViewThatFits(in: .horizontal) {
                        HStack(alignment: .top, spacing: HaneulchiMetrics.Padding.columnGap) {
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

                        VStack(alignment: .leading, spacing: HaneulchiMetrics.Padding.columnGap) {
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
                }
            }
            .padding(.horizontal, HaneulchiMetrics.Padding.page)
            .padding(.vertical, HaneulchiChrome.Spacing.panelGap)
            .padding(.bottom, HaneulchiChrome.Spacing.panelGap)
        }
        .background(HaneulchiChrome.Surface.foundation)
    }
}
