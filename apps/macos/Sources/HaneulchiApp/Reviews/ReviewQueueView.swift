import SwiftUI

struct ReviewQueueView: View {
    let summary: String
    @StateObject private var viewModel: ReviewQueueViewModel

    init(
        summary: String = "Review Queue reads Rust-owned evidence summaries for review-ready tasks.",
        viewModel: ReviewQueueViewModel = ReviewQueueViewModel()
    ) {
        self.summary = summary
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
            VStack(alignment: .leading, spacing: 8) {
                Text("Review Queue")
                    .font(HaneulchiTypography.heading(28))
                Text(summary)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }

            if viewModel.items.isEmpty {
                Text(viewModel.emptyStateMessage)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                    .padding(18)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(HaneulchiChrome.Colors.surfaceMuted)
                    .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))
            } else {
                HStack(alignment: .top, spacing: 20) {
                    VStack(alignment: .leading, spacing: 10) {
                        ForEach(viewModel.items) { item in
                            Button {
                                viewModel.select(taskID: item.taskID)
                            } label: {
                                VStack(alignment: .leading, spacing: 6) {
                                    Text(item.title)
                                        .font(HaneulchiTypography.heading(16))
                                    Text(item.summary)
                                        .font(HaneulchiTypography.caption)
                                        .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                                }
                                .padding(16)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(HaneulchiChrome.Colors.surfaceMuted)
                                .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .frame(width: 320)

                    ReviewSummaryPanelView(item: viewModel.selectedItem) { command in
                        try? viewModel.apply(command)
                    }
                }
            }

            if let degradedReason = viewModel.degradedReason {
                Text("Degraded: \(degradedReason)")
                    .font(HaneulchiTypography.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.warning)
            }
        }
        .padding(HaneulchiChrome.Spacing.screenPadding)
        .background(HaneulchiChrome.Colors.primaryPanel)
        .task {
            try? viewModel.reload()
        }
    }
}
