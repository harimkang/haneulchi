import SwiftUI

struct ReviewQueueView: View {
    let summary: String
    @StateObject private var viewModel: ReviewQueueViewModel

    init(
        summary: String =
            "Review Queue reads Rust-owned evidence summaries for review-ready tasks.",
        viewModel: ReviewQueueViewModel = ReviewQueueViewModel(),
    ) {
        self.summary = summary
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                Text("Review Queue")
                    .font(HaneulchiTypography.display)
                    .foregroundStyle(HaneulchiChrome.Label.primary)
                Text(summary)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
            }

            if viewModel.items.isEmpty {
                Text(viewModel.emptyStateMessage)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
                    .padding(HaneulchiMetrics.Padding.card)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(HaneulchiChrome.Surface.base)
                    .clipShape(RoundedRectangle(
                        cornerRadius: HaneulchiMetrics.Radius.large,
                        style: .continuous,
                    ))
            } else {
                HStack(alignment: .top, spacing: HaneulchiMetrics.Padding.columnGap) {
                    // Left panel: queue list
                    VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                        HaneulchiSectionHeader(
                            title: "Ready for Review",
                            count: viewModel.items.count,
                        )
                        ForEach(viewModel.items) { item in
                            reviewReadyRow(item: item)
                        }
                    }
                    .frame(width: 300)

                    // Center + Right: summary, evidence, decision rail
                    ReviewSummaryPanelView(item: viewModel.selectedItem) { command in
                        do {
                            try viewModel.apply(command)
                        } catch {
                            // The view model stores the operator-visible error state.
                        }
                    }
                }
            }

            if let degradedReason = viewModel.degradedReason {
                HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.system(size: HaneulchiMetrics.Icon.small))
                        .foregroundStyle(HaneulchiChrome.State.error)
                    Text("Degraded: \(degradedReason)")
                        .font(HaneulchiTypography.bodySmall)
                        .foregroundStyle(HaneulchiChrome.State.error)
                }
                .padding(.horizontal, HaneulchiMetrics.Padding.compact)
                .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
                .background(HaneulchiChrome.State.errorSolid.opacity(0.12))
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            }
            if let actionError = viewModel.actionError {
                HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: HaneulchiMetrics.Icon.small))
                        .foregroundStyle(HaneulchiChrome.State.error)
                    Text("Action failed: \(actionError)")
                        .font(HaneulchiTypography.bodySmall)
                        .foregroundStyle(HaneulchiChrome.State.error)
                }
                .padding(.horizontal, HaneulchiMetrics.Padding.compact)
                .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
                .background(HaneulchiChrome.State.errorSolid.opacity(0.12))
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            }
        }
        .padding(HaneulchiChrome.Spacing.screenPadding)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Surface.foundation)
        .task {
            try? viewModel.reload()
        }
    }

    @ViewBuilder
    private func reviewReadyRow(item: ReviewQueueProjectionPayload.Item) -> some View {
        let isSelected = viewModel.selectedTaskID == item.taskID

        Button {
            viewModel.select(taskID: item.taskID)
        } label: {
            HStack(alignment: .top, spacing: HaneulchiMetrics.Spacing.xs) {
                // review_ready accent stripe
                RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.small)
                    .fill(HaneulchiChrome.Gradient.primaryEnd)
                    .frame(width: 3)

                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                    HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.xs) {
                        HaneulchiStatusBadge(state: .reviewReady, label: "REVIEW READY")
                        Spacer()
                    }
                    Text(item.title)
                        .font(HaneulchiTypography.sectionHeading)
                        .foregroundStyle(HaneulchiChrome.Label.primary)
                    Text(item.summary)
                        .font(HaneulchiTypography.caption)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                        .lineLimit(2)
                    if let hookSummary = item.hookSummary {
                        HStack(spacing: HaneulchiMetrics.Spacing.xxs) {
                            Image(systemName: "exclamationmark.triangle.fill")
                                .font(.system(size: HaneulchiMetrics.Icon.small))
                                .foregroundStyle(HaneulchiChrome.State.warning)
                            Text(hookSummary)
                                .font(HaneulchiTypography.caption)
                                .foregroundStyle(HaneulchiChrome.State.warning)
                        }
                    }
                }
            }
            .padding(HaneulchiMetrics.Padding.card)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(isSelected ? HaneulchiChrome.Surface.raised : HaneulchiChrome.Surface.base)
            .clipShape(RoundedRectangle(
                cornerRadius: HaneulchiMetrics.Radius.large,
                style: .continuous,
            ))
            .overlay(
                RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large)
                    .strokeBorder(
                        isSelected ? HaneulchiChrome.Stroke.ghost : Color.clear,
                        lineWidth: 1,
                    ),
            )
        }
        .buttonStyle(.plain)
        .animation(
            .easeInOut(duration: HaneulchiMetrics.Motion.pressedSelection),
            value: isSelected,
        )
    }
}
