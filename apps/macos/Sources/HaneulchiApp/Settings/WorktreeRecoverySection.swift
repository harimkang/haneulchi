import SwiftUI

struct WorktreeRecoverySection: View {
    let issues: [SettingsStatusViewModel.DegradedIssueRow]
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            Text("Worktree & Recovery")
                .font(.title3.weight(.semibold))

            if issues.isEmpty {
                Text("No recovery issues detected.")
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            } else {
                ForEach(issues) { issue in
                    VStack(alignment: .leading, spacing: 4) {
                        ViewThatFits(in: .horizontal) {
                            HStack(alignment: .firstTextBaseline, spacing: 8) {
                                Text(issue.issueCode)
                                    .font(.headline)
                                Text("degraded")
                                    .font(.caption.weight(.semibold))
                                    .foregroundStyle(HaneulchiChrome.Colors.warning)
                            }

                            VStack(alignment: .leading, spacing: 4) {
                                Text(issue.issueCode)
                                    .font(.headline)
                                Text("degraded")
                                    .font(.caption.weight(.semibold))
                                    .foregroundStyle(HaneulchiChrome.Colors.warning)
                            }
                        }
                        Text(issue.details)
                            .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                        Button("Recover") {
                            onAction(.triggerRecovery(issueCode: issue.issueCode))
                        }
                        .buttonStyle(.bordered)
                        .controlSize(.small)
                    }
                    .padding(.vertical, 4)
                }
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.primaryPanel)
        .clipShape(RoundedRectangle(cornerRadius: 18))
    }
}
