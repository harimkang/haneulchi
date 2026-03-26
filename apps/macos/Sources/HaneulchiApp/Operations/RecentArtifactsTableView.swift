import SwiftUI

struct RecentArtifactsTableView: View {
    let items: [ControlTowerViewModel.RecentArtifactItem]
    let onOpen: (ControlTowerViewModel.RecentArtifactItem) -> Void

    var body: some View {
        HaneulchiOpsRailPanel(title: "Recent Artifacts", count: items.isEmpty ? nil : items.count) {
            if items.isEmpty {
                Text("No recent artifacts.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
                    .frame(maxWidth: .infinity, alignment: .leading)
            } else {
                VStack(alignment: .leading, spacing: 0) {
                    headerRow

                    VStack(spacing: 0) {
                        ForEach(items) { item in
                            Button {
                                onOpen(item)
                            } label: {
                                HaneulchiTableRow {
                                    HStack(alignment: .top, spacing: HaneulchiMetrics.Spacing.sm) {
                                        Text(item.taskID)
                                            .font(HaneulchiTypography.compactMeta)
                                            .foregroundStyle(HaneulchiChrome.Gradient.primaryEnd)
                                            .frame(width: 96, alignment: .leading)

                                        Text(item.projectID)
                                            .font(HaneulchiTypography.compactMeta)
                                            .foregroundStyle(HaneulchiChrome.Label.secondary)
                                            .frame(width: 92, alignment: .leading)

                                        Text(item.summary)
                                            .font(HaneulchiTypography.bodySmall)
                                            .foregroundStyle(HaneulchiChrome.Label.primary)
                                            .frame(maxWidth: .infinity, alignment: .leading)
                                            .lineLimit(2)

                                        Text(item.targetRoute.title)
                                            .font(HaneulchiTypography.compactMeta)
                                            .foregroundStyle(HaneulchiChrome.Label.muted)
                                            .frame(width: 88, alignment: .trailing)
                                    }
                                }
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
                .background(HaneulchiChrome.Surface.recess)
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }

    private var headerRow: some View {
        HStack(spacing: HaneulchiMetrics.Spacing.sm) {
            Text("TASK ID")
                .font(HaneulchiTypography.compactMeta)
                .tracking(HaneulchiTypography.Tracking.labelWide)
                .foregroundStyle(HaneulchiChrome.Label.muted)
                .frame(width: 96, alignment: .leading)

            Text("PROJECT")
                .font(HaneulchiTypography.compactMeta)
                .tracking(HaneulchiTypography.Tracking.labelWide)
                .foregroundStyle(HaneulchiChrome.Label.muted)
                .frame(width: 92, alignment: .leading)

            Text("ARTIFACT SUMMARY")
                .font(HaneulchiTypography.compactMeta)
                .tracking(HaneulchiTypography.Tracking.labelWide)
                .foregroundStyle(HaneulchiChrome.Label.muted)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("ROUTE")
                .font(HaneulchiTypography.compactMeta)
                .tracking(HaneulchiTypography.Tracking.labelWide)
                .foregroundStyle(HaneulchiChrome.Label.muted)
                .frame(width: 88, alignment: .trailing)
        }
        .padding(.horizontal, HaneulchiMetrics.Padding.card)
        .padding(.vertical, HaneulchiMetrics.Spacing.xs)
        .background(HaneulchiChrome.Surface.base)
    }
}
