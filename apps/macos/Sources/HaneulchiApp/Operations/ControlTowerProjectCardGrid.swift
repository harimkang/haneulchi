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
                            HStack(alignment: .top) {
                                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                                    Text(card.title)
                                        .font(HaneulchiTypography.sectionHeading)
                                        .foregroundStyle(HaneulchiChrome.Label.primary)
                                    HaneulchiStatusBadge(
                                        state: card.statusBadgeState,
                                        label: card.statusLabel,
                                    )
                                }
                                Spacer()
                                Image(systemName: card.iconName)
                                    .font(.system(size: HaneulchiMetrics.Icon.standard))
                                    .foregroundStyle(HaneulchiChrome.Label.muted)
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

                            visualHeatmap(strip: card.heatStrip)
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

    private func visualHeatmap(strip: ControlTowerViewModel.HeatStrip) -> some View {
        HStack(spacing: 4) {
            let total = strip.running + strip.waitingInput + strip.reviewReady + strip.blocked
            if total == 0 {
                ForEach(0..<7, id: \.self) { _ in
                    RoundedRectangle(cornerRadius: 2)
                        .fill(HaneulchiChrome.Surface.recess)
                        .frame(height: 12)
                }
            } else {
                let baseColor = strip.blocked > 0 ? HaneulchiChrome.State.errorSolid :
                    (strip.waitingInput > 0 || strip.reviewReady > 0) ? HaneulchiChrome.Gradient.primaryEnd :
                    HaneulchiChrome.State.successSolid
                
                let opacities: [Double] = [0.2, 0.4, 0.8, 0.6, 0.9, 0.3, 1.0] // Simulated active density
                
                ForEach(0..<opacities.count, id: \.self) { idx in
                    RoundedRectangle(cornerRadius: 2)
                        .fill(baseColor.opacity(opacities[idx]))
                        .frame(height: 12)
                }
            }
        }
    }
}
