import SwiftUI

struct BottomStatusStripView: View {
    let items: [AppShellChromeState.StripItem]
    let transientNotice: String?
    let onAction: (AppShellAction) -> Void

    var body: some View {
        HStack(spacing: 14) {
            ForEach(items) { item in
                HStack(spacing: 6) {
                    Text(item.title)
                        .font(HaneulchiTypography.label(11))
                    if let detail = item.detail {
                        Text(detail)
                            .font(HaneulchiTypography.caption)
                            .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                    }
                }
            }

            Spacer()

            if let transientNotice {
                Text(transientNotice)
                    .font(HaneulchiTypography.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }
        }
        .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)
        .padding(.vertical, 10)
        .background(HaneulchiChrome.Colors.surfaceRaised)
    }
}
