import SwiftUI

struct TopAppBarView: View {
    let chrome: AppShellChromeState
    let onAction: (AppShellAction) -> Void
    @Environment(\.viewportContext) private var viewportContext

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

            chipCluster

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

    @ViewBuilder
    private var chipCluster: some View {
        switch viewportContext.shellChromeDensity {
        case .regular:
            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                ForEach(chrome.topBarChips) { chip in
                    chipPill(chip)
                }
            }
        case .compact:
            let presentation = viewportContext.compactTopBarChipPresentation(
                for: chrome.topBarChips,
                visibleLimit: 2,
            )
            let chips = presentation.visibleChips + [presentation.overflowChip].compactMap(\.self)

            HStack(spacing: HaneulchiMetrics.Spacing.xxs) {
                ForEach(chips) { chip in
                    chipPill(chip, compact: true)
                }
            }
        }
    }

    private func chipPill(
        _ chip: AppShellChromeState.Chip,
        compact: Bool = false,
    ) -> some View {
        let horizontalPadding = compact ? HaneulchiMetrics.Spacing.xxs : HaneulchiMetrics.Spacing.xs

        return Text(chip.title)
            .font(HaneulchiTypography.compactMeta)
            .foregroundStyle(HaneulchiChrome.Label.secondary)
            .lineLimit(1)
            .minimumScaleFactor(compact ? 0.8 : 1)
            .padding(.horizontal, horizontalPadding)
            .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
            .background(chipBackground(chip))
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.pill))
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
