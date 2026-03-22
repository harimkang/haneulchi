import SwiftUI

struct ReadinessSettingsSection: View {
    let report: ReadinessReport?
    let workflowStatus: WorkflowStatusPayload?

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            Text("Readiness")
                .font(.title3.weight(.semibold))

            if let report, !report.checks.isEmpty {
                ForEach(report.checks, id: \.name) { check in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(check.headline)
                            .font(.headline)
                        Text(check.detail)
                            .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                        if let nextAction = check.nextAction {
                            Text("Next: \(nextAction)")
                                .font(.caption)
                                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                        }
                    }
                    .padding(.vertical, 4)
                }
            } else {
                Text("No readiness report loaded yet.")
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }

            if let workflowStatus {
                Divider()
                VStack(alignment: .leading, spacing: 4) {
                    Text("Workflow")
                        .font(.headline)
                    Text(workflowStatus.path)
                        .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                    Text("State: \(workflowStatus.state.rawValue)")
                        .font(.caption)
                    if let lastError = workflowStatus.lastError {
                        Text(lastError)
                            .font(.caption)
                            .foregroundStyle(HaneulchiChrome.Colors.warning)
                    }
                }
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .background(HaneulchiChrome.Colors.primaryPanel)
        .clipShape(RoundedRectangle(cornerRadius: 18))
    }
}
