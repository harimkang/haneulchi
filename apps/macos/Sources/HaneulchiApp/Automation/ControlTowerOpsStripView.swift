import SwiftUI

struct ControlTowerOpsStripView: View {
    let model: AutomationPanelViewModel
    let onRefresh: (() -> Void)?
    let onReconcile: (() -> Void)?
    let onReload: (() -> Void)?

    init(
        model: AutomationPanelViewModel,
        onRefresh: (() -> Void)? = nil,
        onReconcile: (() -> Void)? = nil,
        onReload: (() -> Void)? = nil,
    ) {
        self.model = model
        self.onRefresh = onRefresh
        self.onReconcile = onReconcile
        self.onReload = onReload
    }

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
            HaneulchiMonolithStrip(metrics: model.primaryStripMetrics) {
                HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                    actionButton(.refresh, action: onRefresh)
                    actionButton(.reconcile, action: onReconcile)
                    actionButton(.reload, action: onReload)
                }
            }

            HStack(spacing: HaneulchiMetrics.Spacing.lg) {
                ForEach(model.secondaryStripMetrics) { metric in
                    HaneulchiMetricTile(metric: metric)
                }
            }
            .padding(.horizontal, HaneulchiMetrics.Padding.card)
        }
    }

    private func actionButton(_ chromeAction: HaneulchiChromeAction, action: (() -> Void)?)
        -> some View
    {
        HaneulchiIconButton(action: chromeAction, tone: .secondary) {
            action?()
        }
        .disabled(action == nil)
    }
}
