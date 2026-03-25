import SwiftUI

struct SessionStackView: View {
    struct Row: Equatable, Identifiable {
        let sessionID: String
        let title: String
        let summary: String
        let branch: String?
        let unreadCount: Int
        let signal: SessionSignalPresentation?
        let isFocused: Bool
        let showsManualContinueCTA: Bool

        var id: String { sessionID }
    }

    let rows: [Row]
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HaneulchiSectionHeader(title: "Sessions", count: rows.isEmpty ? nil : rows.count)

            VStack(alignment: .leading, spacing: 0) {
                ForEach(rows) { row in
                    sessionRow(row)
                }
            }
        }
        .frame(width: HaneulchiMetrics.Panel.explorerMin, alignment: .topLeading)
        .background(HaneulchiChrome.Surface.recess)
    }

    @ViewBuilder
    private func sessionRow(_ row: Row) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Button {
                onAction(.jumpToSession(row.sessionID))
            } label: {
                HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.xs) {
                    // Unread indicator dot
                    if row.unreadCount > 0 {
                        Circle()
                            .fill(HaneulchiChrome.State.warning)
                            .frame(width: 6, height: 6)
                    } else {
                        Circle()
                            .fill(Color.clear)
                            .frame(width: 6, height: 6)
                    }

                    VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                        HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                            Text(row.title)
                                .font(HaneulchiTypography.systemLabel)
                                .tracking(HaneulchiTypography.Tracking.labelWide)
                                .foregroundStyle(
                                    row.isFocused
                                        ? HaneulchiChrome.Label.primary
                                        : HaneulchiChrome.Label.secondary
                                )
                                .lineLimit(1)

                            if let signal = row.signal {
                                HaneulchiStatusBadge(
                                    state: signal.badgeState,
                                    label: signal.label
                                )
                            }

                            Spacer()
                        }

                        if let branch = row.branch {
                            Text(branch)
                                .font(HaneulchiTypography.compactMeta)
                                .tracking(HaneulchiTypography.Tracking.metaModerate)
                                .foregroundStyle(HaneulchiChrome.Label.muted)
                                .lineLimit(1)
                        }

                        Text(row.summary)
                            .font(HaneulchiTypography.compactMeta)
                            .tracking(HaneulchiTypography.Tracking.metaModerate)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                            .lineLimit(1)
                    }

                    if row.isFocused {
                        Rectangle()
                            .fill(HaneulchiChrome.Gradient.primaryEnd)
                            .frame(width: 2)
                            .frame(maxHeight: .infinity)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, HaneulchiMetrics.Padding.compact)
                .padding(.vertical, HaneulchiMetrics.Spacing.xs)
                .frame(minHeight: HaneulchiMetrics.Target.compact)
                .background(
                    row.isFocused
                        ? HaneulchiChrome.Surface.raised
                        : HaneulchiChrome.Surface.recess
                )
            }
            .buttonStyle(.plain)
            .paneAttentionDecoration(
                hasAttention: row.signal?.tone == .strong,
                hasUnread: row.unreadCount > 0
            )

            if row.showsManualContinueCTA {
                Button("Open Session for Manual Continue") {
                    onAction(.jumpToSession(row.sessionID))
                }
                .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                .font(HaneulchiTypography.compactMeta)
                .padding(.horizontal, HaneulchiMetrics.Padding.compact)
                .padding(.bottom, HaneulchiMetrics.Spacing.xs)
            }
        }
    }

    nonisolated static func rows(from snapshot: AppShellSnapshot) -> [Row] {
        snapshot.sessions.map { session in
            let isFocused = session.focusState == .focused || snapshot.app.focusedSessionID == session.sessionID
            return Row(
                sessionID: session.sessionID,
                title: session.title,
                summary: session.latestSummary ?? session.currentDirectory ?? session.title,
                branch: session.branch,
                unreadCount: session.unreadCount,
                signal: SessionSignalPresentation.from(session: session, isFocused: isFocused),
                isFocused: isFocused,
                showsManualContinueCTA: session.canTakeover || session.manualControlState == .takeover
            )
        }
    }
}
