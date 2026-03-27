import SwiftUI

struct ControlTowerOpsStripView: View {
    let model: AutomationPanelViewModel
    let onRefresh: (() -> Void)?
    let onReconcile: (() -> Void)?
    let onReload: (() -> Void)?
    @Environment(\.viewportContext) private var viewportContext

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
                ViewThatFits(in: .horizontal) {
                    HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                        actionButton(.refresh, action: onRefresh)
                        actionButton(.reconcile, action: onReconcile)
                        actionButton(.reload, action: onReload)
                    }

                    VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                        actionButton(.refresh, action: onRefresh)
                        actionButton(.reconcile, action: onReconcile)
                        actionButton(.reload, action: onReload)
                    }
                }
            }

            HaneulchiOpsRailPanel(
                title: "Diagnostics",
                count: model.secondaryStripMetrics.isEmpty ? nil : model.secondaryStripMetrics
                    .count,
            ) {
                LazyVGrid(
                    columns: secondaryMetricColumns,
                    alignment: .leading,
                    spacing: HaneulchiMetrics.Spacing.sm,
                ) {
                    ForEach(model.secondaryStripMetrics) { metric in
                        HaneulchiMetricTile(metric: metric)
                    }
                }
            }
        }
    }

    private var secondaryMetricColumns: [GridItem] {
        let count = switch viewportContext.viewportClass {
        case .compact:
            1
        case .medium:
            2
        case .wide:
            3
        case .expanded:
            4
        }

        return Array(
            repeating: GridItem(.flexible(minimum: 120), spacing: HaneulchiMetrics.Spacing.sm),
            count: count,
        )
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
