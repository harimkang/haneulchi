import SwiftUI

struct ReviewQueueView: View {
    let summary: String
    @StateObject private var viewModel: ReviewQueueViewModel
    @Environment(\.viewportContext) private var viewportContext
    private let layout = HaneulchiOperationalLayoutMetrics.standard

    private var responsiveLayout: ReviewQueueResponsiveLayout {
        .init(viewportClass: viewportContext.viewportClass)
    }

    init(
        summary: String =
            "Review Queue reads Rust-owned evidence summaries for review-ready tasks.",
        viewModel: ReviewQueueViewModel = ReviewQueueViewModel(),
    ) {
        self.summary = summary
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    var body: some View {
        Group {
            if responsiveLayout.requiresVerticalOverflowScroll {
                ScrollView {
                    routeContent
                }
            } else {
                routeContent
            }
        }
        .background(HaneulchiChrome.Surface.foundation)
        .task {
            try? viewModel.reload()
        }
    }

    private var routeContent: some View {
        VStack(alignment: .leading, spacing: layout.sectionSpacing) {
            HaneulchiHeaderDeck(
                title: "Review Queue",
                subtitle: summary,
                horizontalPadding: layout.headerInnerPadding,
            ) {
                EmptyView()
            }

            if viewModel.items.isEmpty {
                HaneulchiOpsRailPanel(title: "Ready for Review") {
                    Text(viewModel.emptyStateMessage)
                        .font(HaneulchiTypography.body)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } else {
                switch responsiveLayout.mode {
                case .stacked:
                    VStack(alignment: .leading, spacing: layout.columnSpacing) {
                        reviewListPanel
                        detailPanel
                    }
                case .split:
                    HStack(alignment: .top, spacing: layout.columnSpacing) {
                        splitScrollingPane {
                            reviewListPanel
                        }
                        .frame(
                            width: responsiveLayout.masterColumnWidth,
                            alignment: .topLeading,
                        )

                        splitScrollingPane {
                            detailPanel
                        }
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                }
            }
        }
        .padding(.horizontal, layout.screenPadding)
        .padding(.vertical, layout.sectionSpacing)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    @ViewBuilder
    private func splitScrollingPane(@ViewBuilder content: () -> some View) -> some View {
        if responsiveLayout.usesIndependentPaneScrolling {
            ScrollView {
                content()
                    .frame(maxWidth: .infinity, alignment: .topLeading)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        } else {
            content()
        }
    }

    @ViewBuilder
    private func reviewReadyRow(item: ReviewQueueProjectionPayload.Item) -> some View {
        let isSelected = viewModel.selectedTaskID == item.taskID

        Button {
            viewModel.select(taskID: item.taskID)
        } label: {
            HaneulchiSignalRow(
                accent: item.surfaceAccent,
                eyebrow: "REVIEW READY",
                title: item.title,
                summary: item.summary,
                meta: "\(item.projectID) · \(item.warningSummary)",
            ) {
                VStack(alignment: .trailing, spacing: HaneulchiMetrics.Spacing.xxs) {
                    HaneulchiStatusBadge(
                        state: item.warnings.isEmpty ? .reviewReady : .blocked,
                        label: item.warningSummary.uppercased(),
                    )

                    if let hookSummary = item.hookSummary {
                        Text(hookSummary)
                            .font(HaneulchiTypography.compactMeta)
                            .foregroundStyle(HaneulchiChrome.State.warning)
                            .multilineTextAlignment(.trailing)
                    }
                }
            }
            .overlay(
                RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium)
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

    private var reviewListPanel: some View {
        HaneulchiOpsRailPanel(
            title: "Ready for Review",
            count: viewModel.items.count,
        ) {
            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                ForEach(viewModel.items) { item in
                    reviewReadyRow(item: item)
                }
            }
        }
    }

    private var detailPanel: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
            if let degradedReason = viewModel.degradedReason {
                statusBanner(
                    icon: "exclamationmark.triangle.fill",
                    message: "Degraded: \(degradedReason)",
                )
            }
            if let actionError = viewModel.actionError {
                statusBanner(
                    icon: "xmark.circle.fill",
                    message: "Action failed: \(actionError)",
                )
            }

            ReviewSummaryPanelView(item: viewModel.selectedItem) { command in
                do {
                    try viewModel.apply(command)
                } catch {
                    // The view model stores the operator-visible error state.
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }

    private func statusBanner(icon: String, message: String) -> some View {
        HStack(spacing: HaneulchiMetrics.Spacing.xs) {
            Image(systemName: icon)
                .font(.system(size: HaneulchiMetrics.Icon.small))
                .foregroundStyle(HaneulchiChrome.State.error)
            Text(message)
                .font(HaneulchiTypography.bodySmall)
                .foregroundStyle(HaneulchiChrome.State.error)
        }
        .padding(.horizontal, HaneulchiMetrics.Padding.compact)
        .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
        .background(HaneulchiChrome.State.errorSolid.opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }
}
