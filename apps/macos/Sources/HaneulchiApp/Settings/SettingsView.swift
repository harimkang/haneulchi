import SwiftUI

struct SettingsView: View {
    let report: ReadinessReport?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                Text("Settings")
                    .font(.largeTitle.weight(.bold))
                ReadinessSettingsSection(report: report)
            }
            .padding(HaneulchiChrome.Spacing.screenPadding)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
        .background(HaneulchiChrome.Colors.appBackground)
    }
}
