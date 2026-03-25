import SwiftUI

struct AttentionCenterView: View {
    let viewModel: AttentionCenterViewModel

    private var groupedItems: [AttentionCenterPresentation.GroupedItems] {
        AttentionCenterPresentation.grouped(viewModel.items)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                    Text("Attention Center")
                        .font(HaneulchiTypography.display)
                        .foregroundStyle(HaneulchiChrome.Label.primary)
                        .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)

                    if viewModel.items.isEmpty {
                        Text("No active attention items.")
                            .font(HaneulchiTypography.body)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                            .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)
                    }
                }

                ForEach(groupedItems, id: \.group.rawValue) { section in
                    VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                        HaneulchiSectionHeader(title: section.group.displayTitle, count: section.items.count)
                            .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)

                        ForEach(section.items) { item in
                            attentionItemRow(item: item, group: section.group)
                                .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)
                        }
                    }
                }
            }
            .padding(.vertical, HaneulchiChrome.Spacing.panelGap)
        }
        .background(HaneulchiChrome.Surface.foundation)
    }

    @ViewBuilder
    private func attentionItemRow(
        item: AttentionCenterViewModel.Item,
        group: AttentionCenterPresentation.Group
    ) -> some View {
        HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.sm) {
            // State chip
            HaneulchiStatusBadge(state: group.badgeState, label: group.badgeLabel)

            // Content
            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                Text(item.headline)
                    .font(HaneulchiTypography.sectionHeading)
                    .foregroundStyle(HaneulchiChrome.Label.primary)
                if let summary = item.summary {
                    Text(summary)
                        .font(HaneulchiTypography.bodySmall)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                        .lineLimit(2)
                }
                // State label as timestamp-style meta
                Text(item.stateLabel)
                    .font(HaneulchiTypography.compactMeta)
                    .tracking(HaneulchiTypography.Tracking.metaModerate)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            // Action buttons
            VStack(alignment: .trailing, spacing: HaneulchiMetrics.Spacing.xxs) {
                Button("Open") { viewModel.open(item) }
                    .buttonStyle(HaneulchiButtonStyle(variant: .primary))

                HStack(spacing: HaneulchiMetrics.Spacing.xxs) {
                    Button("Resolve") { viewModel.resolve(item) }
                        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                    Button("Dismiss") { viewModel.dismiss(item) }
                        .buttonStyle(HaneulchiButtonStyle(variant: .tertiary))
                    Button("Snooze") { viewModel.snooze(item) }
                        .buttonStyle(HaneulchiButtonStyle(variant: .tertiary))
                }
            }
        }
        .padding(HaneulchiMetrics.Padding.card)
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large, style: .continuous))
    }
}
