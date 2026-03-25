import SwiftUI

struct BottomStatusStripView: View {
    let items: [AppShellChromeState.StripItem]
    let transientNotice: String?
    let onAction: (AppShellAction) -> Void

    var body: some View {
        HStack(spacing: HaneulchiMetrics.Spacing.sm) {
            ForEach(items) { item in
                HStack(spacing: HaneulchiMetrics.Spacing.xxs) {
                    Text(item.title)
                        .font(HaneulchiTypography.compactMeta)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                    if let detail = item.detail {
                        Text(detail)
                            .font(HaneulchiTypography.compactMeta)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                    }
                }
            }

            Spacer()

            if let transientNotice {
                Text(transientNotice)
                    .font(HaneulchiTypography.compactMeta)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
            }
        }
        .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)
        .frame(height: HaneulchiMetrics.Shell.bottomStripHeight)
        .background(HaneulchiChrome.Surface.recess)
    }
}
