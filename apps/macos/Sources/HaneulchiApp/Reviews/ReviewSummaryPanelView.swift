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
        VStack(alignment: .leading, spacing: 12) {
            Text("Review Summary")
                .font(HaneulchiTypography.heading(18))

            if let item {
                Text(item.title)
                    .font(HaneulchiTypography.heading(20))
                ReviewEvidencePackView(model: ReviewEvidencePackModel(item: item))

                TaskTimelineSection(title: "Audit Timeline", entries: item.timeline)

                HStack(spacing: 10) {
                    actionButton("Accept", command: .accept, prominent: true)
                    actionButton("Request Changes", command: .requestChanges, prominent: false)
                    actionButton("Manual Continue", command: .manualContinue, prominent: false)
                    actionButton("Create Follow-up", command: .followUp, prominent: false)
                }
            } else {
                Text("Select a review-ready task to inspect its evidence summary.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }
        }
        .padding(18)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.surfaceRaised)
        .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))
    }

    private func actionButton(
        _ title: String,
        command: ReviewDecisionCommand,
        prominent: Bool
    ) -> some View {
        Group {
            if prominent {
                Button(title) {
                    onDecision?(command)
                }
                .buttonStyle(BorderedProminentButtonStyle())
            } else {
                Button(title) {
                    onDecision?(command)
                }
                .buttonStyle(BorderedButtonStyle())
            }
        }
        .disabled(item == nil || onDecision == nil)
    }
}
