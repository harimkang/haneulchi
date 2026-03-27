import SwiftUI

struct SettingsView: View {
    let viewModel: SettingsStatusViewModel
    let onAction: (AppShellAction) -> Void
    @Environment(\.viewportContext) private var viewportContext

    private var sectionColumns: [GridItem] {
        Array(
            repeating: GridItem(.flexible(), spacing: HaneulchiChrome.Spacing.panelGap),
            count: viewportContext.viewportClass >= .wide ? 2 : 1,
        )
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                Text("Settings")
                    .font(.largeTitle.weight(.bold))

                LazyVGrid(columns: sectionColumns, alignment: .leading, spacing: HaneulchiChrome
                    .Spacing.panelGap)
                {
                    ReadinessSettingsSection(viewModel: viewModel)
                    TerminalSettingsSection(row: viewModel.terminalSettingsRow)
                    SecretsSettingsSection()
                    WorktreeRecoverySection(
                        issues: viewModel.degradedIssueRows,
                        onAction: onAction,
                    )
                    SettingsAutomationStatusSection(viewModel: viewModel, onAction: onAction)
                }
            }
            .padding(HaneulchiChrome.Spacing.screenPadding)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
        .background(HaneulchiChrome.Colors.appBackground)
    }
}
