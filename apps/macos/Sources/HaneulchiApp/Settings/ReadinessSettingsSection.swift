import SwiftUI

struct ReadinessSettingsSection: View {
    let viewModel: SettingsStatusViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            Text("Readiness")
                .font(.title3.weight(.semibold))

            if !viewModel.readinessRows.isEmpty {
                ForEach(viewModel.readinessRows) { row in
                    rowBlock(
                        headline: row.headline,
                        detail: row.detail,
                        statusLabel: row.statusLabel,
                        nextAction: row.nextAction
                    )
                    .padding(.vertical, 4)
                }
            } else {
                Text("No readiness report loaded yet.")
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }

            if let shellIntegrationRow = viewModel.shellIntegrationRow {
                rowBlock(
                    headline: shellIntegrationRow.headline,
                    detail: shellIntegrationRow.detail,
                    statusLabel: shellIntegrationRow.statusLabel,
                    nextAction: shellIntegrationRow.nextAction
                )
                .padding(.top, 8)
            }

            if !viewModel.presetRows.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Presets")
                        .font(.headline)
                    ForEach(viewModel.presetRows) { preset in
                        VStack(alignment: .leading, spacing: 2) {
                            HStack(alignment: .firstTextBaseline, spacing: 8) {
                                Text(preset.title)
                                Text(preset.statusLabel)
                                    .font(.caption.weight(.semibold))
                                    .foregroundStyle(statusColor(for: preset.statusLabel))
                            }
                            Text(preset.detail)
                                .font(.caption)
                                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                            if preset.requiresShellIntegration {
                                Text("Requires shell integration")
                                    .font(.caption)
                                    .foregroundStyle(HaneulchiChrome.Colors.warning)
                            }
                        }
                        .padding(.vertical, 2)
                    }
                }
                .padding(.top, 8)
            }

            if let workflowRow = viewModel.workflowRow {
                VStack(alignment: .leading, spacing: 4) {
                    HStack(alignment: .firstTextBaseline, spacing: 8) {
                        Text(workflowRow.title)
                            .font(.headline)
                        Text(workflowRow.statusLabel)
                            .font(.caption)
                            .foregroundStyle(statusColor(for: workflowRow.statusLabel))
                    }
                    Text(workflowRow.detail)
                        .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                    if let lastError = workflowRow.lastError {
                        Text(lastError)
                            .font(.caption)
                            .foregroundStyle(HaneulchiChrome.Colors.warning)
                    }
                }
                .padding(.top, 8)
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .background(HaneulchiChrome.Colors.primaryPanel)
        .clipShape(RoundedRectangle(cornerRadius: 18))
    }

    @ViewBuilder
    private func rowBlock(
        headline: String,
        detail: String,
        statusLabel: String,
        nextAction: String?
    ) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(headline)
                    .font(.headline)
                Text(statusLabel)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(statusColor(for: statusLabel))
            }
            Text(detail)
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            if let nextAction {
                Text("Next: \(nextAction)")
                    .font(.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }
        }
    }

    private func statusColor(for label: String) -> Color {
        switch label {
        case "ready", "available", "installed", "ok":
            HaneulchiChrome.Colors.ready
        case "blocked":
            HaneulchiChrome.Colors.blocked
        default:
            HaneulchiChrome.Colors.warning
        }
    }
}
