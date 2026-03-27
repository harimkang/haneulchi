import SwiftUI

struct ControlTowerProjectCardGrid: View {
    let cards: [ControlTowerViewModel.ProjectCard]
    let onOpenProject: (String) -> Void
    @Environment(\.viewportContext) private var viewportContext
    @State private var availableWidth: CGFloat = 0
    private let layout = HaneulchiOperationalLayoutMetrics.standard

    private var gridLayout: ControlTowerProjectGridLayout {
        ControlTowerResponsiveLayout(viewportClass: viewportContext.viewportClass)
            .projectGridLayout(
                availableWidth: availableWidth,
                spacing: layout.gridSpacing,
            )
    }

    private var columns: [GridItem] {
        Array(
            repeating: GridItem(
                .fixed(gridLayout.cardWidth),
                spacing: layout.gridSpacing,
                alignment: .topLeading,
            ),
            count: gridLayout.columnCount,
        )
    }

    var body: some View {
        LazyVGrid(columns: columns, alignment: .leading, spacing: layout.gridSpacing) {
            ForEach(cards) { card in
                Button {
                    onOpenProject(card.projectID)
                } label: {
                    HaneulchiCard {
                        HStack(alignment: .top, spacing: HaneulchiMetrics.Spacing.sm) {
                            RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.small)
                                .fill(card.accent.line)
                                .frame(width: 4)

                            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                                HStack(alignment: .top, spacing: HaneulchiMetrics.Spacing.sm) {
                                    VStack(
                                        alignment: .leading,
                                        spacing: HaneulchiMetrics.Spacing.xxs,
                                    ) {
                                        HaneulchiStatusBadge(
                                            state: card.statusBadgeState,
                                            label: card.statusLabel.uppercased(),
                                        )
                                        Text(card.title)
                                            .font(HaneulchiTypography.sectionHeading)
                                            .foregroundStyle(HaneulchiChrome.Label.primary)
                                    }

                                    Spacer()

                                    Image(systemName: card.iconName)
                                        .font(.system(size: HaneulchiMetrics.Icon.standard))
                                        .foregroundStyle(HaneulchiChrome.Label.muted)
                                }

                                ViewThatFits(in: .horizontal) {
                                    HStack(spacing: HaneulchiMetrics.Spacing.md) {
                                        ForEach(card.overviewMetrics, id: \.label) { metric in
                                            overviewMetric(metric)
                                        }
                                        Spacer(minLength: HaneulchiMetrics.Spacing.sm)
                                        if let spotlight = card.spotlightLabel {
                                            metaPill(spotlight)
                                        }
                                    }

                                    VStack(
                                        alignment: .leading,
                                        spacing: HaneulchiMetrics.Spacing.xs,
                                    ) {
                                        HStack(spacing: HaneulchiMetrics.Spacing.md) {
                                            ForEach(card.overviewMetrics, id: \.label) { metric in
                                                overviewMetric(metric)
                                            }
                                        }

                                        if let spotlight = card.spotlightLabel {
                                            metaPill(spotlight)
                                        }
                                    }
                                }

                                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                                    Text(card.latestSummary ?? "No recent summary")
                                        .font(HaneulchiTypography.body)
                                        .foregroundStyle(HaneulchiChrome.Label.primary)
                                        .fixedSize(horizontal: false, vertical: true)

                                    if let commentary = card.latestCommentary {
                                        Text(commentary)
                                            .font(HaneulchiTypography.compactMeta)
                                            .foregroundStyle(HaneulchiChrome.Label.muted)
                                            .lineLimit(2)
                                    }
                                }

                                visualHeatmap(strip: card.heatStrip, accent: card.accent)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
                .buttonStyle(.plain)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .onGeometryChange(for: CGFloat.self) { geometry in
            geometry.size.width
        } action: { _, width in
            availableWidth = width
        }
    }

    private func overviewMetric(_ metric: ControlTowerViewModel.ProjectCard
        .OverviewMetric) -> some View
    {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
            Text(metric.label.uppercased())
                .font(HaneulchiTypography.compactMeta)
                .tracking(HaneulchiTypography.Tracking.labelWide)
                .foregroundStyle(HaneulchiChrome.Label.muted)
            Text(metric.value)
                .font(HaneulchiTypography.bodySmall)
                .foregroundStyle(HaneulchiChrome.Label.primary)
        }
    }

    private func metaPill(_ value: String) -> some View {
        Text(value)
            .font(HaneulchiTypography.compactMeta)
            .tracking(HaneulchiTypography.Tracking.metaModerate)
            .foregroundStyle(HaneulchiChrome.Label.secondary)
            .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
            .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
            .background(HaneulchiChrome.Surface.recess)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.pill))
    }

    private func visualHeatmap(
        strip: ControlTowerViewModel.HeatStrip,
        accent: HaneulchiSignalAccent,
    ) -> some View {
        HStack(spacing: 4) {
            let total = strip.running + strip.waitingInput + strip.reviewReady + strip.blocked
            if total == 0 {
                ForEach(0 ..< 7, id: \.self) { _ in
                    RoundedRectangle(cornerRadius: 2)
                        .fill(HaneulchiChrome.Surface.recess)
                        .frame(height: 12)
                }
            } else {
                let opacities: [Double] = [0.2, 0.4, 0.8, 0.6, 0.9, 0.3, 1.0]

                ForEach(0 ..< opacities.count, id: \.self) { idx in
                    RoundedRectangle(cornerRadius: 2)
                        .fill(accent.line.opacity(opacities[idx]))
                        .frame(height: 12)
                }
            }
        }
    }
}
