import SwiftUI

struct ControlTowerView: View {
    let model: ControlTowerViewModel
    let onAction: (AppShellAction) -> Void

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                HStack {
                    Spacer()
                    Button("Quick Dispatch") {
                        onAction(.presentQuickDispatch(.controlTower))
                    }
                    .buttonStyle(.bordered)
                }
                ControlTowerOpsStripView(
                    model: model.opsModel,
                    onRefresh: { onAction(.refreshShellSnapshot) },
                    onReconcile: { onAction(.reconcileAutomation) },
                    onReload: { onAction(.reloadWorkflow) }
                )

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
            .padding(.bottom, HaneulchiChrome.Spacing.panelGap)
        }
        .background(HaneulchiChrome.Colors.appBackground)
    }
}
