import SwiftUI

struct AutomationControlPanelView: View {
    let viewModel: AutomationControlPanelViewModel
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("WF-17 Control Panel")
                .font(HaneulchiTypography.heading(18))
            Text(viewModel.orchestratorSummary)
                .font(HaneulchiTypography.caption)
            Text(viewModel.workflowSummary)
                .font(HaneulchiTypography.caption)
            Text(viewModel.apiSummary)
                .font(HaneulchiTypography.caption)
            Text(viewModel.cliSummary)
                .font(HaneulchiTypography.caption)
            Text("tracker: \(viewModel.trackerSummary)")
                .font(HaneulchiTypography.caption)
            HStack {
                ForEach(viewModel.actions, id: \.self) { title in
                    Button(title) {
                        onAction(action(for: title))
                    }
                    .buttonStyle(.bordered)
                    .font(HaneulchiTypography.label(11))
                }
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(HaneulchiChrome.Colors.surfaceRaised)
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
    }

    private func action(for title: String) -> AppShellAction {
        switch title {
        case "Refresh":
            .refreshShellSnapshot
        case "Reconcile":
            .reconcileAutomation
        case "Reload Workflow":
            .reloadWorkflow
        case "Export Snapshot":
            .exportSnapshot
        default:
            .refreshShellSnapshot
        }
    }
}
