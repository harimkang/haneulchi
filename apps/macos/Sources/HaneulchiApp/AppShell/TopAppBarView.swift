import SwiftUI

struct TopAppBarView: View {
    let chrome: AppShellChromeState
    let onAction: (AppShellAction) -> Void

    var body: some View {
        HStack(spacing: HaneulchiMetrics.Spacing.sm) {
            VStack(alignment: .leading, spacing: 2) {
                Text("Haneulchi")
                    .font(HaneulchiTypography.compactMeta)
                    .foregroundStyle(HaneulchiChrome.Label.muted)

                Text(chrome.topBarTitle)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.primary)
            }

            Spacer()

            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                ForEach(chrome.topBarChips) { chip in
                    Text(chip.title)
                        .font(HaneulchiTypography.compactMeta)
                        .foregroundStyle(HaneulchiChrome.Label.secondary)
                        .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
                        .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
                        .background(chipBackground(chip))
                        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.pill))
                }
            }

            Button("Command Palette") {
                onAction(.toggleCommandPalette)
            }
            .font(HaneulchiTypography.systemLabel)
            .foregroundStyle(HaneulchiChrome.Label.secondary)
            .buttonStyle(.plain)
            .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
            .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
            .background(HaneulchiChrome.Surface.base)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))

            Button("Notifications") {
                onAction(.toggleNotificationDrawer)
            }
            .font(HaneulchiTypography.systemLabel)
            .foregroundStyle(HaneulchiChrome.Label.secondary)
            .buttonStyle(.plain)
            .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
            .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
            .background(HaneulchiChrome.Surface.base)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
        }
        .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)
        .frame(height: HaneulchiMetrics.Shell.topBarHeight)
        .background(.ultraThinMaterial)
        .background(HaneulchiChrome.Surface.foundation.opacity(0.72))
    }

    private func chipBackground(_ chip: AppShellChromeState.Chip) -> Color {
        switch chip.tone {
        case .degraded:
            HaneulchiChrome.State.warning.opacity(0.18)
        case .failed:
            HaneulchiChrome.State.error.opacity(0.18)
        case .unread:
            HaneulchiChrome.State.warning.opacity(0.18)
        case nil:
            HaneulchiChrome.Surface.base
        }
    }
}
