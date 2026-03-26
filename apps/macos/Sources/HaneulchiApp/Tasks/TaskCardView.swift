import SwiftUI

struct TaskCardView: View {
    let task: TaskBoardProjectionPayload.TaskCard
    @Environment(\.viewportContext) private var viewportContext

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
                        label: task.column.title,
                    )
                }

                Text(task.title)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.primary)
                    .lineLimit(viewportContext.viewportClass == .compact ? 4 : 3)

                metadataChips

                if task.column == .running {
                    runningHeartbeatStrip
                }

                Text(task.contextSummaryLabel)
                    .font(HaneulchiTypography.compactMeta)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
                    .lineLimit(viewportContext.viewportClass <= .medium ? 2 : 1)

                nextActionRow
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
                },
            )
    }

    private func compactChip(_ label: String) -> some View {
        Text(label)
            .font(HaneulchiTypography.compactMeta)
            .foregroundStyle(HaneulchiChrome.Label.secondary)
            .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
            .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
            .background(HaneulchiChrome.Surface.recess)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.pill))
    }

    private var metadataChips: some View {
        ViewThatFits(in: .horizontal) {
            HStack(spacing: HaneulchiMetrics.Spacing.xxs) {
                ForEach(task.compactMetadataChips, id: \.self) { chip in
                    compactChip(chip)
                }
            }

            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                ForEach(task.compactMetadataChips, id: \.self) { chip in
                    compactChip(chip)
                }
            }
        }
    }

    private var nextActionRow: some View {
        ViewThatFits(in: .horizontal) {
            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                nextActionLabel
                nextActionValue
            }

            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                nextActionLabel
                nextActionValue
            }
        }
    }

    private var nextActionLabel: some View {
        Text("next")
            .font(HaneulchiTypography.compactMeta)
            .foregroundStyle(HaneulchiChrome.Label.muted)
    }

    private var nextActionValue: some View {
        Text(task.nextActionLabel)
            .font(HaneulchiTypography.bodySmall)
            .foregroundStyle(HaneulchiChrome.Label.secondary)
            .lineLimit(viewportContext.viewportClass <= .medium ? 2 : 1)
    }

    private func badgeState(for column: TaskBoardColumnID) -> HaneulchiStatusBadge.State {
        switch column {
        case .inbox: .idle
        case .ready: .idle
        case .running: .active
        case .review: .reviewReady
        case .blocked: .blocked
        case .done: .done
        }
    }
}
