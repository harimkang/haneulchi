import SwiftUI

struct BottomStatusStripView: View {
    let items: [AppShellChromeState.StripItem]
    let transientNotice: String?
    let onAction: (AppShellAction) -> Void
    @Environment(\.viewportContext) private var viewportContext

    var body: some View {
        stripContent
            .padding(.horizontal, HaneulchiChrome.Spacing.densePadding)
            .frame(height: HaneulchiMetrics.Shell.bottomStripHeight)
            .background(HaneulchiChrome.Surface.recess)
            .overlay(
                Rectangle()
                    .frame(height: 1)
                    .foregroundColor(HaneulchiChrome.Stroke.ghost),
                alignment: .top,
            )
    }

    @ViewBuilder
    private var stripContent: some View {
        switch viewportContext.shellChromeDensity {
        case .regular:
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
        case .compact:
            let presentation = viewportContext.compactBottomStripPresentation(
                items: items,
                transientNotice: transientNotice,
            )

            HStack(alignment: .firstTextBaseline, spacing: HaneulchiMetrics.Spacing.sm) {
                HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                    ForEach(presentation.items) { item in
                        Text(compactLabel(for: item))
                            .font(HaneulchiTypography.compactMeta)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                            .lineLimit(1)
                            .minimumScaleFactor(0.8)
                    }
                }
                .layoutPriority(1)

                Spacer(minLength: HaneulchiMetrics.Spacing.sm)

                if let transientNotice = presentation.transientNotice {
                    Text(transientNotice)
                        .font(HaneulchiTypography.compactMeta)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                        .lineLimit(1)
                        .truncationMode(.tail)
                        .minimumScaleFactor(0.75)
                        .layoutPriority(0)
                }
            }
        }
    }

    private func compactLabel(for item: AppShellChromeState.StripItem) -> String {
        guard let detail = item.detail else {
            return item.title
        }

        return "\(item.title) · \(detail)"
    }
}
