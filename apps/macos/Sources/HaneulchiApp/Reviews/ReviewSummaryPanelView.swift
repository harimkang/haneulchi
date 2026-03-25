import SwiftUI

struct ReviewSummaryPanelView: View {
    let item: ReviewQueueProjectionPayload.Item?
    let onDecision: ((ReviewDecisionCommand) -> Void)?

    init(
        item: ReviewQueueProjectionPayload.Item?,
        onDecision: ((ReviewDecisionCommand) -> Void)? = nil
    ) {
        self.item = item
        self.onDecision = onDecision
    }

    var body: some View {
        HStack(alignment: .top, spacing: HaneulchiMetrics.Padding.columnGap) {
            // Center: summary + evidence
            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                HaneulchiSectionHeader(title: "Review Summary")

                if let item {
                    HaneulchiPanel {
                        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                            // Title + completeness badge
                            HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.xs) {
                                Text(item.title)
                                    .font(HaneulchiTypography.sectionHeading)
                                    .foregroundStyle(HaneulchiChrome.Label.primary)
                                Spacer()
                                let badgeState: HaneulchiStatusBadge.State = item.warnings.isEmpty ? .active : .blocked
                                let badgeLabel = item.warnings.isEmpty ? "COMPLETE" : "\(item.warnings.count) WARNINGS"
                                HaneulchiStatusBadge(state: badgeState, label: badgeLabel)
                            }

                            // Summary text
                            Text(item.summary)
                                .font(HaneulchiTypography.body)
                                .foregroundStyle(HaneulchiChrome.Label.primary)
                                .fixedSize(horizontal: false, vertical: true)

                            // Warning items
                            if !item.warnings.isEmpty {
                                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
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
                                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
                            }
                        }
                    }

                    ReviewEvidencePackView(model: ReviewEvidencePackModel(item: item))

                    TaskTimelineSection(title: "Audit Timeline", entries: item.timeline)
                } else {
                    HaneulchiPanel {
                        Text("Select a review-ready task to inspect its evidence summary.")
                            .font(HaneulchiTypography.body)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .topLeading)

            // Right: decision rail
            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                HaneulchiSectionHeader(title: "Decision")

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
                .padding(HaneulchiMetrics.Padding.card)
                .background(HaneulchiChrome.Surface.base)
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large))
            }
            .frame(width: 200, alignment: .topLeading)
        }
    }
}
