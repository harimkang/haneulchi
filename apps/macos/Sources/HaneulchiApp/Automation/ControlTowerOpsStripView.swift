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
                    actionButton("Refresh", action: onRefresh)
                    actionButton("Reconcile", action: onReconcile)
                    actionButton("Reload", action: onReload)
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

    private func actionButton(_ title: String, action: (() -> Void)?) -> some View {
        Button(title) {
            action?()
        }
        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
        .disabled(action == nil)
    }
}
