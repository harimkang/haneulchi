import SwiftUI

struct RecentArtifactsTableView: View {
    let items: [ControlTowerViewModel.RecentArtifactItem]
    let onOpen: (ControlTowerViewModel.RecentArtifactItem) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Recent Artifacts")
                .font(HaneulchiTypography.heading(18))

            if items.isEmpty {
                Text("No recent artifacts.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            } else {
                ForEach(items) { item in
                    Button {
                        onOpen(item)
                    } label: {
                        HStack(alignment: .top) {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(item.taskID)
                                    .font(HaneulchiTypography.label(11))
                                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                                Text(item.summary)
                                    .font(HaneulchiTypography.body)
                                if let manifestPath = item.manifestPath {
                                    Text(manifestPath)
                                        .font(HaneulchiTypography.caption)
                                        .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                                }
                            }
                            Spacer()
                            Text(item.targetRoute.title)
                                .font(HaneulchiTypography.label(11))
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.vertical, 8)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .background(HaneulchiChrome.Colors.surfaceBase)
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
    }
}
