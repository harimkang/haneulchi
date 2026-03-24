import SwiftUI

struct SettingsAutomationStatusSection: View {
    let viewModel: SettingsStatusViewModel
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            Text("Automation & Integrations")
                .font(.title3.weight(.semibold))

            ForEach(viewModel.automationRows) { row in
                VStack(alignment: .leading, spacing: 4) {
                    HStack(alignment: .firstTextBaseline, spacing: 8) {
                        Text(row.title)
                            .font(.headline)
                        Text(row.statusLabel)
                            .font(.caption.weight(.semibold))
                            .foregroundStyle(statusColor(for: row.statusLabel))
                    }
                    Text(row.detail)
                        .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                    if let nextAction = row.nextAction {
                        Text("Next: \(nextAction)")
                            .font(.caption)
                            .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                    }
                }
                .padding(.vertical, 4)
            }

            if let controlPanel = viewModel.controlPanel {
                AutomationControlPanelView(viewModel: controlPanel, onAction: onAction)
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .background(HaneulchiChrome.Colors.surfaceMuted)
        .clipShape(RoundedRectangle(cornerRadius: 18))
    }

    private func statusColor(for label: String) -> Color {
        switch label {
        case "available", "ready":
            HaneulchiChrome.Colors.ready
        case "blocked":
            HaneulchiChrome.Colors.blocked
        default:
            HaneulchiChrome.Colors.warning
        }
    }
}
