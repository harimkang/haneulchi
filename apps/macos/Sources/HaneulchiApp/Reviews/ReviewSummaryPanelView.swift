import SwiftUI

struct ReviewSummaryPanelView: View {
    let item: ReviewQueueProjectionPayload.Item?

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Review Summary")
                .font(HaneulchiTypography.heading(18))

            if let item {
                Text(item.title)
                    .font(HaneulchiTypography.heading(20))
                Text(item.summary)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)

                if !item.touchedFiles.isEmpty {
                    Text("Touched files: \(item.touchedFiles.joined(separator: ", "))")
                        .font(HaneulchiTypography.caption)
                }
                if let diffSummary = item.diffSummary {
                    Text("Diff: \(diffSummary)")
                        .font(HaneulchiTypography.caption)
                }
                if let testsSummary = item.testsSummary {
                    Text("Tests: \(testsSummary)")
                        .font(HaneulchiTypography.caption)
                }
                if let commandSummary = item.commandSummary {
                    Text("Commands: \(commandSummary)")
                        .font(HaneulchiTypography.caption)
                }
                if !item.warnings.isEmpty {
                    Text("Warnings: \(item.warnings.joined(separator: ", "))")
                        .font(HaneulchiTypography.caption)
                        .foregroundStyle(HaneulchiChrome.Colors.warning)
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
}
