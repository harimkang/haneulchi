import SwiftUI

struct AttentionCenterView: View {
    let viewModel: AttentionCenterViewModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
                Text("Attention Center")
                    .font(HaneulchiTypography.heading(28))
                    .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)

                ForEach(viewModel.items) { item in
                    VStack(alignment: .leading, spacing: 10) {
                        Text(item.headline)
                            .font(HaneulchiTypography.heading(18))
                        if let summary = item.summary {
                            Text(summary)
                                .font(HaneulchiTypography.body)
                                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                        }
                        HStack(spacing: 8) {
                            Button("Open") { viewModel.open(item) }
                            Button("Resolve") { viewModel.resolve(item) }
                            Button("Dismiss") { viewModel.dismiss(item) }
                            Button("Snooze") { viewModel.snooze(item) }
                        }
                        .buttonStyle(.bordered)
                    }
                    .padding(HaneulchiChrome.Spacing.panelPadding)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(HaneulchiChrome.Colors.surfaceBase)
                    .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
                    .padding(.horizontal, HaneulchiChrome.Spacing.screenPadding)
                }
            }
            .padding(.vertical, HaneulchiChrome.Spacing.panelGap)
        }
        .background(HaneulchiChrome.Colors.appBackground)
    }
}
