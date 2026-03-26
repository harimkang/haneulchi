import SwiftUI

struct ReviewSummaryPanelView: View {
    let item: ReviewQueueProjectionPayload.Item?
    let onDecision: ((ReviewDecisionCommand) -> Void)?
    @Environment(\.viewportContext) private var viewportContext
    private let layout = HaneulchiOperationalLayoutMetrics.standard

    private var responsiveLayout: ReviewQueueResponsiveLayout {
        .init(viewportClass: viewportContext.viewportClass)
    }

    init(
        item: ReviewQueueProjectionPayload.Item?,
        onDecision: ((ReviewDecisionCommand) -> Void)? = nil,
    ) {
        self.item = item
        self.onDecision = onDecision
    }

    var body: some View {
        Group {
            if responsiveLayout.stacksDecisionPanel {
                VStack(alignment: .leading, spacing: layout.columnSpacing) {
                    summaryPanel
                    decisionPanel
                }
            } else {
                HStack(alignment: .top, spacing: layout.columnSpacing) {
                    summaryPanel
                    decisionPanel
                        .frame(width: layout.decisionRailWidth, alignment: .topLeading)
                }
            }
        }
    }

    private var summaryPanel: some View {
        HaneulchiOpsRailPanel(title: "Review Summary") {
            if let item {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                    HaneulchiPanel {
                        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                            ViewThatFits(in: .horizontal) {
                                HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.xs) {
                                    summaryTitle(item.title)
                                    Spacer(minLength: HaneulchiMetrics.Spacing.xs)
                                    warningBadge(for: item)
                                }

                                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                                    summaryTitle(item.title)
                                    warningBadge(for: item)
                                }
                            }

                            Text(item.summary)
                                .font(HaneulchiTypography.body)
                                .foregroundStyle(HaneulchiChrome.Label.primary)
                                .fixedSize(horizontal: false, vertical: true)

                            if !item.warnings.isEmpty {
                                VStack(
                                    alignment: .leading,
                                    spacing: HaneulchiMetrics.Spacing.xxs,
                                ) {
                                    ForEach(item.warnings, id: \.self) { warning in
                                        HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                                            Image(systemName: "exclamationmark.triangle.fill")
                                                .font(.system(size: HaneulchiMetrics.Icon.small))
                                                .foregroundStyle(HaneulchiChrome.State.error)
                                            Text(warning)
                                                .font(HaneulchiTypography.bodySmall)
                                                .foregroundStyle(HaneulchiChrome.State.error)
                                        }
                                    }
                                }
                                .padding(HaneulchiMetrics.Padding.compact)
                                .background(HaneulchiChrome.State.errorSolid.opacity(0.10))
                                .clipShape(
                                    RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium),
                                )
                            }
                        }
                    }

                    ReviewEvidencePackView(model: ReviewEvidencePackModel(item: item))

                    TaskTimelineSection(title: "Audit Timeline", entries: item.timeline)
                }
            } else {
                Text("Select a review-ready task to inspect its evidence summary.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }

    private var decisionPanel: some View {
        HaneulchiOpsRailPanel(title: "Decision") {
            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                decisionButton("Accept", variant: .primary, command: .accept)
                decisionButton("Request Changes", variant: .secondary, command: .requestChanges)
                decisionButton("Create Follow-up", variant: .secondary, command: .followUp)
                decisionButton("Manual Continue", variant: .tertiary, command: .manualContinue)
            }
        }
    }

    private func summaryTitle(_ value: String) -> some View {
        Text(value)
            .font(HaneulchiTypography.sectionHeading)
            .foregroundStyle(HaneulchiChrome.Label.primary)
    }

    private func warningBadge(for item: ReviewQueueProjectionPayload.Item) -> some View {
        HaneulchiStatusBadge(
            state: item.warnings.isEmpty ? .reviewReady : .blocked,
            label: item.warningSummary.uppercased(),
        )
    }

    private func decisionButton(
        _ title: String,
        variant: HaneulchiButtonStyle.Variant,
        command: ReviewDecisionCommand,
    ) -> some View {
        Button(title) { onDecision?(command) }
            .buttonStyle(HaneulchiButtonStyle(variant: variant))
            .disabled(item == nil || onDecision == nil)
            .frame(maxWidth: .infinity)
    }
}
