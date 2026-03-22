import SwiftUI

struct TaskTimelineEntry: Codable, Equatable, Identifiable, Sendable {
    let id: String
    let kind: String
    let actor: String
    let summary: String
    let warningReason: String?
    let createdAt: String

    enum CodingKeys: String, CodingKey {
        case id
        case kind
        case actor
        case summary
        case warningReason = "warning_reason"
        case createdAt = "created_at"
    }
}

struct TaskTimelineSection: View {
    let title: String
    let entries: [TaskTimelineEntry]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(HaneulchiTypography.label(13))
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)

            if entries.isEmpty {
                Text("No timeline yet.")
                    .font(HaneulchiTypography.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            } else {
                ForEach(entries) { entry in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(entry.summary)
                            .font(HaneulchiTypography.body)
                        Text("\(entry.kind) · \(entry.actor) · \(entry.createdAt)")
                            .font(HaneulchiTypography.caption)
                            .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                        if let warningReason = entry.warningReason {
                            Text("warning: \(warningReason)")
                                .font(HaneulchiTypography.caption)
                                .foregroundStyle(HaneulchiChrome.Colors.warning)
                        }
                    }
                    .padding(.vertical, 6)
                }
            }
        }
    }
}
