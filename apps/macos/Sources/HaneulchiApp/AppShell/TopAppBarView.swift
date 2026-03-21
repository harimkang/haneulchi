import SwiftUI

struct TopAppBarView: View {
    let chrome: AppShellChromeState
    let onAction: (AppShellAction) -> Void

    var body: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Haneulchi")
                    .font(HaneulchiTypography.label(12))
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)

                Text(chrome.topBarTitle)
                    .font(HaneulchiTypography.heading(20).weight(.bold))
                    .foregroundStyle(.primary)
            }

            Spacer()

            HStack(spacing: 8) {
                ForEach(chrome.topBarChips) { chip in
                    Text(chip.title)
                        .font(HaneulchiTypography.label(11))
                        .padding(.horizontal, 10)
                        .padding(.vertical, 5)
                        .background(chipBackground(chip))
                        .clipShape(Capsule())
                }
            }

            Button("Command Palette") {
                onAction(.toggleCommandPalette)
            }
            .buttonStyle(.bordered)
        }
        .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)
        .padding(.vertical, 16)
        .background(HaneulchiChrome.Colors.surfaceRaised)
    }

    private func chipBackground(_ chip: AppShellChromeState.Chip) -> Color {
        switch chip.tone {
        case .degraded:
            HaneulchiChrome.Colors.warning.opacity(0.18)
        case .failed:
            HaneulchiChrome.Colors.blocked.opacity(0.18)
        case .unread:
            HaneulchiChrome.Colors.unread.opacity(0.18)
        case nil:
            HaneulchiChrome.Colors.surfaceMuted
        }
    }
}
