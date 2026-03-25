import SwiftUI

struct ControlTowerProjectCardGrid: View {
    let cards: [ControlTowerViewModel.ProjectCard]
    let onOpenProject: (String) -> Void

    private let columns = [
        GridItem(.flexible(minimum: 220), spacing: HaneulchiChrome.Spacing.itemGap),
        GridItem(.flexible(minimum: 220), spacing: HaneulchiChrome.Spacing.itemGap),
    ]

    var body: some View {
        LazyVGrid(columns: columns, spacing: HaneulchiChrome.Spacing.itemGap) {
            ForEach(cards) { card in
                Button {
                    onOpenProject(card.projectID)
                } label: {
                    HaneulchiCard {
                        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                            HStack(alignment: .firstTextBaseline) {
                                Text(card.title)
                                    .font(HaneulchiTypography.sectionHeading)
                                    .foregroundStyle(HaneulchiChrome.Label.primary)
                                Spacer()
                                HaneulchiStatusBadge(
                                    state: card.statusBadgeState,
                                    label: card.statusLabel,
                                )
                            }

                            HStack(spacing: HaneulchiMetrics.Spacing.md) {
                                metaItem("sessions", card.sessionCountLabel)
                                metaItem("attention", card.attentionCountLabel)
                            }

                            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                                Text(card.latestSummary ?? "No recent summary")
                                    .font(HaneulchiTypography.body)
                                    .foregroundStyle(HaneulchiChrome.Label.primary)
                                if let commentary = card.latestCommentary {
                                    Text(commentary)
                                        .font(HaneulchiTypography.compactMeta)
                                        .foregroundStyle(HaneulchiChrome.Label.muted)
                                }
                            }

                            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                                heatChip("run", card.heatStrip.running)
                                heatChip("wait", card.heatStrip.waitingInput)
                                heatChip("review", card.heatStrip.reviewReady)
                                heatChip("block", card.heatStrip.blocked)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
                .buttonStyle(.plain)
            }
        }
    }

    private func metaItem(_ label: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
            Text(label)
                .font(HaneulchiTypography.compactMeta)
                .foregroundStyle(HaneulchiChrome.Label.muted)
            Text(value)
                .font(HaneulchiTypography.compactMeta)
                .foregroundStyle(HaneulchiChrome.Label.muted)
        }
    }

    private func heatChip(_ label: String, _ value: Int) -> some View {
        Text("\(label) \(value)")
            .font(HaneulchiTypography.compactMeta)
            .foregroundStyle(HaneulchiChrome.Label.muted)
            .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
            .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
            .background(HaneulchiChrome.Surface.recess)
            .clipShape(Capsule())
    }
}
