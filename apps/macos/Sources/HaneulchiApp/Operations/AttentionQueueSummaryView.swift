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
                        state: items.count > 0 ? .waitingInput : .idle,
                    )
                }

                // Item list
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                    ForEach(items) { item in
                        HStack(spacing: 0) {
                            // Target indicator border
                            Rectangle()
                                .fill(item.targetRoute == .reviewQueue ? HaneulchiChrome.Gradient
                                    .primaryEnd : HaneulchiChrome.State.warningSolid)
                                .frame(width: 4)

                            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                                HStack(alignment: .firstTextBaseline) {
                                    Text(item
                                        .targetRoute == .reviewQueue ? "REVIEW READY" :
                                        "WAITING INPUT")
                                        .font(HaneulchiTypography.compactMeta)
                                        .tracking(HaneulchiTypography.Tracking.labelWide)
                                        .foregroundStyle(item
                                            .targetRoute == .reviewQueue ? HaneulchiChrome.Gradient
                                            .primaryEnd : HaneulchiChrome.State.warning)
                                        .bold()
                                    Spacer()
                                    Text("Now")
                                        .font(HaneulchiTypography.compactMeta)
                                        .foregroundStyle(HaneulchiChrome.Label.muted)
                                }

                                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                                    Text(item.headline)
                                        .font(HaneulchiTypography.body)
                                        .foregroundStyle(HaneulchiChrome.Label.primary)
                                        .fixedSize(horizontal: false, vertical: true)
                                    if let summary = item.summary {
                                        Text(summary)
                                            .font(HaneulchiTypography.compactMeta)
                                            .tracking(HaneulchiTypography.Tracking.metaModerate)
                                            .foregroundStyle(HaneulchiChrome.Label.muted)
                                            .lineLimit(2)
                                    }
                                }

                                HStack {
                                    Button {
                                        onOpen(item)
                                    } label: {
                                        Text(item
                                            .targetRoute == .reviewQueue ? "LAUNCH REVIEW" :
                                            "RESOLVE")
                                            .frame(maxWidth: .infinity)
                                    }
                                    .buttonStyle(HaneulchiButtonStyle(variant: item
                                            .targetRoute == .reviewQueue ? .primary : .secondary))
                                }
                            }
                            .padding(HaneulchiMetrics.Padding.card)
                            .frame(maxWidth: .infinity, alignment: .leading)
                        }
                        .background(HaneulchiChrome.Surface.raised)
                        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
                    }
                }
            }
        }
        .padding(HaneulchiMetrics.Padding.card)
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(
            cornerRadius: HaneulchiMetrics.Radius.large,
            style: .continuous,
        ))
    }
}
