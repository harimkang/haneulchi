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
        onReload: (() -> Void)? = nil
    ) {
        self.model = model
        self.onRefresh = onRefresh
        self.onReconcile = onReconcile
        self.onReload = onReload
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Control Tower")
                .font(HaneulchiTypography.heading(28))

            HStack(spacing: 18) {
                metric("cadence", model.cadenceLabel)
                metric("last tick", model.lastTickLabel)
                metric("next tick", model.nextTickLabel)
                metric("reconcile", model.lastReconcileLabel)
                metric("slots", model.slotLabel)
                metric("workflow", model.workflowHealth)
                metric("tracker", model.trackerHealth)
                metric("queue", model.queueLabel)
                metric("paused", model.paused ? "yes" : "no")
            }

            HStack(spacing: 10) {
                actionButton("Refresh", action: onRefresh)
                actionButton("Reconcile", action: onReconcile)
                actionButton("Reload", action: onReload)
            }
        }
        .padding(HaneulchiChrome.Spacing.screenPadding)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(HaneulchiChrome.Colors.primaryPanel)
    }

    private func metric(_ label: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .font(HaneulchiTypography.label(12))
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            Text(value)
                .font(HaneulchiTypography.heading(16))
        }
        .padding(.vertical, 10)
        .padding(.horizontal, 12)
        .background(HaneulchiChrome.Colors.surfaceMuted)
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
    }

    private func actionButton(_ title: String, action: (() -> Void)?) -> some View {
        Button(title) {
            action?()
        }
        .buttonStyle(BorderedButtonStyle())
        .disabled(action == nil)
    }
}
