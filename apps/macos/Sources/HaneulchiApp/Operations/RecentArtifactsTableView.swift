import SwiftUI

struct RecentArtifactsTableView: View {
    let items: [ControlTowerViewModel.RecentArtifactItem]
    let onOpen: (ControlTowerViewModel.RecentArtifactItem) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Table header on recess band
            HStack {
                Text("TASK")
                    .font(HaneulchiTypography.systemLabel)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
                    .frame(maxWidth: .infinity, alignment: .leading)
                Text("ROUTE")
                    .font(HaneulchiTypography.systemLabel)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
            }
            .padding(.horizontal, HaneulchiMetrics.Padding.card)
            .padding(.vertical, HaneulchiMetrics.Spacing.xs)
            .background(HaneulchiChrome.Surface.recess)

            if items.isEmpty {
                Text("No recent artifacts.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
                    .padding(.horizontal, HaneulchiMetrics.Padding.card)
                    .padding(.vertical, HaneulchiMetrics.Spacing.md)
            } else {
                VStack(spacing: 0) {
                    ForEach(items) { item in
                        Button {
                            onOpen(item)
                        } label: {
                            HaneulchiTableRow {
                                HStack(alignment: .top) {
                                    VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                                        Text(item.taskID)
                                            .font(HaneulchiTypography.compactMeta)
                                            .foregroundStyle(HaneulchiChrome.Label.muted)
                                        Text(item.summary)
                                            .font(HaneulchiTypography.body)
                                            .foregroundStyle(HaneulchiChrome.Label.primary)
                                        if let manifestPath = item.manifestPath {
                                            Text(manifestPath)
                                                .font(HaneulchiTypography.compactMeta)
                                                .foregroundStyle(HaneulchiChrome.Label.muted)
                                        }
                                    }
                                    Spacer()
                                    Text(item.targetRoute.title)
                                        .font(HaneulchiTypography.compactMeta)
                                        .foregroundStyle(HaneulchiChrome.Label.muted)
                                }
                                .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }
                        .buttonStyle(.plain)
                    }
                }
            }
        }
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large, style: .continuous))
    }
}
