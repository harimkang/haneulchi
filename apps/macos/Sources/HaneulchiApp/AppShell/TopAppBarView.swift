import SwiftUI

struct TopAppBarView: View {
    let chrome: AppShellChromeState
    let onAction: (AppShellAction) -> Void

    var body: some View {
        HStack(spacing: HaneulchiMetrics.Spacing.sm) {
            VStack(alignment: .leading, spacing: 0) {
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
                        .lineLimit(1)
                        .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
                        .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
                        .background(chipBackground(chip))
                        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.pill))
                }
            }

            HaneulchiIconButton(action: .commandPalette, tone: .secondary) {
                onAction(.toggleCommandPalette)
            }

            HaneulchiIconButton(action: .notifications, tone: .secondary) {
                onAction(.toggleNotificationDrawer)
            }
        }
        .padding(.horizontal, HaneulchiChrome.Spacing.densePadding)
        .frame(height: HaneulchiMetrics.Shell.topBarHeight)
        .background(.ultraThinMaterial)
        .background(HaneulchiChrome.Surface.foundation.opacity(0.72))
        .overlay(
            Rectangle()
                .frame(height: 1)
                .foregroundColor(HaneulchiChrome.Stroke.ghost),
            alignment: .bottom,
        )
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
