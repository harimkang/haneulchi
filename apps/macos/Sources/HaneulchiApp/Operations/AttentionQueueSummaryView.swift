import SwiftUI

struct AttentionQueueSummaryView: View {
    let items: [ControlTowerViewModel.AttentionItem]
    let onOpen: (ControlTowerViewModel.AttentionItem) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
            HaneulchiSectionHeader(title: "Attention", count: items.isEmpty ? nil : items.count)

            if items.isEmpty {
                HaneulchiPanel {
                    Text("No active attention items.")
                        .font(HaneulchiTypography.body)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } else {
                // Summary metrics row
                HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                    HaneulchiMetricTile(
                        label: "Total",
                        value: "\(items.count)",
                        state: items.count > 0 ? .waitingInput : .idle
                    )
                }

                // Item list
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(items) { item in
                        Button {
                            onOpen(item)
                        } label: {
                            HaneulchiTableRow {
                                HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.xs) {
                                    HaneulchiStatusBadge(state: .waitingInput, label: "ATTN")

                                    VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                                        Text(item.headline)
                                            .font(HaneulchiTypography.body)
                                            .foregroundStyle(HaneulchiChrome.Label.primary)
                                        if let summary = item.summary {
                                            Text(summary)
                                                .font(HaneulchiTypography.compactMeta)
                                                .tracking(HaneulchiTypography.Tracking.metaModerate)
                                                .foregroundStyle(HaneulchiChrome.Label.muted)
                                                .lineLimit(1)
                                        }
                                    }
                                    .frame(maxWidth: .infinity, alignment: .leading)

                                    Image(systemName: "chevron.right")
                                        .font(.system(size: HaneulchiMetrics.Icon.small))
                                        .foregroundStyle(HaneulchiChrome.Label.muted)
                                }
                            }
                        }
                        .buttonStyle(.plain)
                    }
                }
                .background(HaneulchiChrome.Surface.base)
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            }
        }
        .padding(HaneulchiMetrics.Padding.card)
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large, style: .continuous))
    }
}
