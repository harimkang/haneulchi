import SwiftUI

struct ReviewSummaryPanelView: View {
    let item: ReviewQueueProjectionPayload.Item?
    let onDecision: ((ReviewDecisionCommand) -> Void)?
    private let layout = HaneulchiOperationalLayoutMetrics.standard

    init(
        item: ReviewQueueProjectionPayload.Item?,
        onDecision: ((ReviewDecisionCommand) -> Void)? = nil,
    ) {
        self.item = item
        self.onDecision = onDecision
    }

    var body: some View {
        HStack(alignment: .top, spacing: layout.columnSpacing) {
            HaneulchiOpsRailPanel(title: "Review Summary") {
                if let item {
                    VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                        HaneulchiPanel {
                            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                                HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.xs) {
                                    Text(item.title)
                                        .font(HaneulchiTypography.sectionHeading)
                                        .foregroundStyle(HaneulchiChrome.Label.primary)
                                    Spacer()
                                    HaneulchiStatusBadge(
                                        state: item.warnings.isEmpty ? .reviewReady : .blocked,
                                        label: item.warningSummary.uppercased(),
                                    )
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
                                                    .font(.system(size: HaneulchiMetrics.Icon
                                                            .small))
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
                                        RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius
                                            .medium),
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

            HaneulchiOpsRailPanel(title: "Decision") {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                    Button("Accept") { onDecision?(.accept) }
                        .buttonStyle(HaneulchiButtonStyle(variant: .primary))
                        .disabled(item == nil || onDecision == nil)
                        .frame(maxWidth: .infinity)

                    Button("Request Changes") { onDecision?(.requestChanges) }
                        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                        .disabled(item == nil || onDecision == nil)
                        .frame(maxWidth: .infinity)

                    Button("Create Follow-up") { onDecision?(.followUp) }
                        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                        .disabled(item == nil || onDecision == nil)
                        .frame(maxWidth: .infinity)

                    Button("Manual Continue") { onDecision?(.manualContinue) }
                        .buttonStyle(HaneulchiButtonStyle(variant: .tertiary))
                        .disabled(item == nil || onDecision == nil)
                        .frame(maxWidth: .infinity)
                }
            }
            .frame(width: layout.decisionRailWidth, alignment: .topLeading)
        }
    }
}
