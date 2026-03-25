import SwiftUI

struct TaskCardView: View {
    let task: TaskBoardProjectionPayload.TaskCard

    var body: some View {
        HaneulchiCard {
            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                HStack(alignment: .firstTextBaseline) {
                    Text(task.displayKey)
                        .font(HaneulchiTypography.compactMeta)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                    Spacer()
                    HaneulchiStatusBadge(
                        state: badgeState(for: task.column),
                        label: task.column.title
                    )
                }

                Text(task.title)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.primary)
                    .lineLimit(2)

                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                    metaRow("status", task.column.title)
                    metaRow("owner", task.linkedSessionID ?? "unassigned")
                    metaRow("priority", task.priority.uppercased())
                    metaRow("evidence", task.evidenceReadinessLabel)
                    metaRow("next", task.nextActionLabel)
                    metaRow("automation", task.automationMode.label)
                    metaRow("project", task.projectID)
                }

                if task.column == .running {
                    runningHeartbeatStrip
                }

                metaRow("session", task.linkedSessionID ?? "none")
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private var runningHeartbeatStrip: some View {
        RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.small)
            .fill(HaneulchiChrome.State.success.opacity(0.18))
            .frame(maxWidth: .infinity)
            .frame(height: 3)
            .overlay(
                GeometryReader { geo in
                    RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.small)
                        .fill(HaneulchiChrome.State.success.opacity(0.5))
                        .frame(width: geo.size.width * 0.6)
                }
            )
    }

    private func metaRow(_ label: String, _ value: String) -> some View {
        HStack(spacing: HaneulchiMetrics.Spacing.xs) {
            Text(label)
                .font(HaneulchiTypography.compactMeta)
                .foregroundStyle(HaneulchiChrome.Label.muted)
            Text(value)
                .font(HaneulchiTypography.compactMeta)
                .foregroundStyle(HaneulchiChrome.Label.muted)
        }
    }

    private func badgeState(for column: TaskBoardColumnID) -> HaneulchiStatusBadge.State {
        switch column {
        case .inbox: return .idle
        case .ready: return .idle
        case .running: return .active
        case .review: return .reviewReady
        case .blocked: return .blocked
        case .done: return .done
        }
    }
}
