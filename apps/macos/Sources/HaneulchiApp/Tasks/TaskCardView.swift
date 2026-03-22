import SwiftUI

struct TaskCardView: View {
    let task: TaskBoardProjectionPayload.TaskCard

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .top) {
                Text(task.displayKey)
                    .font(HaneulchiTypography.label(11))
                    .foregroundStyle(HaneulchiChrome.Colors.accent)
                Spacer()
                Text(task.priority.uppercased())
                    .font(HaneulchiTypography.label(10))
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }

            Text(task.title)
                .font(HaneulchiTypography.heading(15))
                .foregroundStyle(.primary)
                .lineLimit(2)

            HStack(spacing: 8) {
                Text(task.projectID)
                    .font(HaneulchiTypography.label(11))
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                Text(task.automationMode.label)
                    .font(HaneulchiTypography.label(11))
                    .foregroundStyle(HaneulchiChrome.Colors.ready)
            }

            if let linkedSessionID = task.linkedSessionID {
                Text("linked session: \(linkedSessionID)")
                    .font(HaneulchiTypography.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(HaneulchiChrome.Colors.surfaceRaised.opacity(0.95))
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
    }
}
