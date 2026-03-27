import SwiftUI

struct AttentionQueueSummaryView: View {
    let items: [ControlTowerViewModel.AttentionItem]
    let onOpen: (ControlTowerViewModel.AttentionItem) -> Void
    @Environment(\.viewportContext) private var viewportContext

    var body: some View {
        HaneulchiOpsRailPanel(title: "Attention Queue", count: items.isEmpty ? nil : items.count) {
            if items.isEmpty {
                Text("No active attention items.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
                    .frame(maxWidth: .infinity, alignment: .leading)
            } else {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                    ForEach(items) { item in
                        attentionCard(item)
                    }
                }
            }
        }
        .frame(
            maxWidth: viewportContext.viewportClass >= .wide ? 320 : .infinity,
            alignment: .topLeading,
        )
    }

    private func attentionCard(_ item: ControlTowerViewModel.AttentionItem) -> some View {
        let accent: HaneulchiSignalAccent = item
            .targetRoute == .reviewQueue ? .reviewReady : .warning
        let actionTitle = item.targetRoute == .reviewQueue ? "Launch Review" : "Resolve"

        return VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
            HStack(alignment: .firstTextBaseline) {
                Text(item.targetRoute == .reviewQueue ? "REVIEW READY" : "WAITING INPUT")
                    .font(HaneulchiTypography.compactMeta)
                    .tracking(HaneulchiTypography.Tracking.labelWide)
                    .foregroundStyle(accent.tint)
                Spacer()
                Text(item.targetRoute.title)
                    .font(HaneulchiTypography.compactMeta)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
            }

            Text(item.headline)
                .font(HaneulchiTypography.sectionHeading)
                .foregroundStyle(HaneulchiChrome.Label.primary)

            if let summary = item.summary {
                Text(summary)
                    .font(HaneulchiTypography.bodySmall)
                    .foregroundStyle(HaneulchiChrome.Label.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Button {
                onOpen(item)
            }
            label: {
                Text(actionTitle)
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(
                HaneulchiButtonStyle(
                    variant: item.targetRoute == .reviewQueue ? .primary : .secondary,
                ),
            )
        }
        .padding(HaneulchiMetrics.Padding.card)
        .background(HaneulchiChrome.Surface.raised)
        .overlay(alignment: .leading) {
            RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.small)
                .fill(accent.line)
                .frame(width: 4)
                .padding(.vertical, HaneulchiMetrics.Padding.compact)
        }
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }
}
