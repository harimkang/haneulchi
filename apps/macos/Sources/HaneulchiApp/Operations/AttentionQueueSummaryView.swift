import SwiftUI

struct AttentionQueueSummaryView: View {
    let items: [ControlTowerViewModel.AttentionItem]
    let onOpen: (ControlTowerViewModel.AttentionItem) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Attention")
                .font(HaneulchiTypography.heading(18))

            if items.isEmpty {
                Text("No active attention items.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            } else {
                ForEach(items) { item in
                    Button {
                        onOpen(item)
                    } label: {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(item.headline)
                                .font(HaneulchiTypography.body)
                            if let summary = item.summary {
                                Text(summary)
                                    .font(HaneulchiTypography.caption)
                                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                            }
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
