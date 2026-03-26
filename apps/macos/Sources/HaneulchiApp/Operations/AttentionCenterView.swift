import SwiftUI

struct AttentionCenterView: View {
    let viewModel: AttentionCenterViewModel

    private var groupedItems: [AttentionCenterPresentation.GroupedItems] {
        AttentionCenterPresentation.grouped(viewModel.items)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                HaneulchiHeaderDeck(
                    title: "Attention Center",
                    subtitle: "Handle manual takeover, failed flows, degraded state, and unread work in priority order.",
                ) {
                    EmptyView()
                }

                if viewModel.items.isEmpty {
                    HaneulchiOpsRailPanel(title: "Queue") {
                        Text("No active attention items.")
                            .font(HaneulchiTypography.body)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } else {
                    ForEach(groupedItems, id: \.group.rawValue) { section in
                        HaneulchiOpsRailPanel(
                            title: section.group.displayTitle,
                            count: section.items.count,
                        ) {
                            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                                ForEach(section.items) { item in
                                    attentionItemRow(item: item, group: section.group)
                                }
                            }
                        }
                    }
                }
            }
            .padding(.horizontal, HaneulchiMetrics.Padding.page)
            .padding(.vertical, HaneulchiChrome.Spacing.panelGap)
        }
        .background(HaneulchiChrome.Surface.foundation)
    }

    private func attentionItemRow(
        item: AttentionCenterViewModel.Item,
        group: AttentionCenterPresentation.Group,
    ) -> some View {
        HaneulchiSignalRow(
            accent: group.accent,
            eyebrow: group.badgeLabel,
            title: item.headline,
            summary: item.summary,
            meta: "\(item.stateLabel) · \(item.targetRoute.title)",
        ) {
            VStack(alignment: .trailing, spacing: HaneulchiMetrics.Spacing.xxs) {
                Button("Open") { viewModel.open(item) }
                    .buttonStyle(HaneulchiButtonStyle(variant: .primary))

                HStack(spacing: HaneulchiMetrics.Spacing.xxs) {
                    HaneulchiIconButton(action: .resolve, tone: .secondary) {
                        viewModel.resolve(item)
                    }
                    HaneulchiIconButton(action: .dismiss, tone: .tertiary) {
                        viewModel.dismiss(item)
                    }
                    HaneulchiIconButton(action: .snooze, tone: .tertiary) {
                        viewModel.snooze(item)
                    }
                }
            }
        }
    }
}
