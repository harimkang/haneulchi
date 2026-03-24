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
                    VStack(alignment: .leading, spacing: 12) {
                        HStack {
                            Text(card.title)
                                .font(HaneulchiTypography.heading(18))
                            Spacer()
                            Text(card.statusLabel)
                                .font(HaneulchiTypography.label(11))
                                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                        }

                        HStack(spacing: 12) {
                            metric("sessions", card.sessionCountLabel)
                            metric("attention", card.attentionCountLabel)
                        }

                        VStack(alignment: .leading, spacing: 6) {
                            Text(card.latestSummary ?? "No recent summary")
                                .font(HaneulchiTypography.body)
                            if let commentary = card.latestCommentary {
                                Text(commentary)
                                    .font(HaneulchiTypography.caption)
                                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                            }
                        }

                        HStack(spacing: 8) {
                            heatChip("run", card.heatStrip.running)
                            heatChip("wait", card.heatStrip.waitingInput)
                            heatChip("review", card.heatStrip.reviewReady)
                            heatChip("block", card.heatStrip.blocked)
                        }
                    }
                    .padding(HaneulchiChrome.Spacing.panelPadding)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(HaneulchiChrome.Colors.surfaceRaised)
                    .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
                }
                .buttonStyle(.plain)
            }
        }
    }

    private func metric(_ label: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .font(HaneulchiTypography.label(11))
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            Text(value)
                .font(HaneulchiTypography.body)
        }
    }

    private func heatChip(_ label: String, _ value: Int) -> some View {
        Text("\(label) \(value)")
            .font(HaneulchiTypography.label(11))
            .padding(.vertical, 6)
            .padding(.horizontal, 8)
            .background(HaneulchiChrome.Colors.surfaceMuted)
            .clipShape(Capsule())
    }
}
