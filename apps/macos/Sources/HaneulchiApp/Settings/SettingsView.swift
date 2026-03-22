import SwiftUI

struct SettingsView: View {
    let report: ReadinessReport?
    let workflowStatus: WorkflowStatusPayload?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                Text("Settings")
                    .font(.largeTitle.weight(.bold))
                ReadinessSettingsSection(report: report, workflowStatus: workflowStatus)
            }
            .padding(HaneulchiChrome.Spacing.screenPadding)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
        .background(HaneulchiChrome.Colors.appBackground)
    }
}
