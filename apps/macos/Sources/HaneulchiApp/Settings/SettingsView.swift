import SwiftUI

struct SettingsView: View {
    let viewModel: SettingsStatusViewModel
    let onAction: (AppShellAction) -> Void

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                Text("Settings")
                    .font(.largeTitle.weight(.bold))
                ReadinessSettingsSection(viewModel: viewModel)
                SettingsAutomationStatusSection(viewModel: viewModel, onAction: onAction)
            }
            .padding(HaneulchiChrome.Spacing.screenPadding)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
        .background(HaneulchiChrome.Colors.appBackground)
    }
}
