import SwiftUI

struct AttentionCenterView: View {
    let viewModel: AttentionCenterViewModel
    @Environment(\.viewportContext) private var viewportContext
    private let layout = HaneulchiOperationalLayoutMetrics.standard

    private var groupedItems: [AttentionCenterPresentation.GroupedItems] {
        AttentionCenterPresentation.grouped(viewModel.items)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: layout.sectionSpacing) {
                HaneulchiHeaderDeck(
                    title: "Attention Center",
                    subtitle: "Handle manual takeover, failed flows, degraded state, and unread work in priority order.",
                    horizontalPadding: layout.headerInnerPadding,
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
            .padding(.horizontal, layout.screenPadding)
            .padding(.vertical, layout.sectionSpacing)
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
            meta: "\(item.stateLabel) · \(item.targetRouteTitle)",
        ) {
            ViewThatFits(in: .horizontal) {
                HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.sm) {
                    openButton(for: item)
                    secondaryActions(for: item)
                }

                VStack(
                    alignment: viewportContext.viewportClass == .compact ? .leading : .trailing,
                    spacing: HaneulchiMetrics.Spacing.xxs,
                ) {
                    openButton(for: item)
                    secondaryActions(for: item)
                }
            }
        }
    }

    private func openButton(for item: AttentionCenterViewModel.Item) -> some View {
        Button("Open") { viewModel.open(item) }
            .buttonStyle(HaneulchiButtonStyle(variant: .primary))
    }

    @ViewBuilder
    private func secondaryActions(for item: AttentionCenterViewModel.Item) -> some View {
        if item.attentionActionID != nil {
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
