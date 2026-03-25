import SwiftUI

struct SecretsSettingsSection: View {
    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            Text("Secrets")
                .font(.title3.weight(.semibold))

            VStack(alignment: .leading, spacing: 6) {
                Text("Manage Keychain-backed secret refs")
                    .font(.headline)
                Text(
                    "Secrets are stored in the macOS Keychain. Only labels and Keychain key references are shown here — secret values are never displayed.",
                )
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                .font(.body)
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .background(HaneulchiChrome.Colors.primaryPanel)
        .clipShape(RoundedRectangle(cornerRadius: 18))
    }
}
