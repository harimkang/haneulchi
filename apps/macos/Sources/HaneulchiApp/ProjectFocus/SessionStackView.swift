import SwiftUI

struct SessionStackView: View {
    enum LayoutStyle: Equatable, Sendable {
        case column
        case compactAffordance
    }

    struct Row: Equatable, Identifiable {
        let sessionID: String
        let title: String
        let summary: String
        let branch: String?
        let unreadCount: Int
        let signal: SessionSignalPresentation?
        let isFocused: Bool
        let showsManualContinueCTA: Bool

        var id: String {
            sessionID
        }
    }

    let rows: [Row]
    var columnWidth: CGFloat = HaneulchiMetrics.Panel.sessionStackWidth
    var layoutStyle: LayoutStyle = .column
    let onAction: (AppShellAction) -> Void

    var body: some View {
        switch layoutStyle {
        case .column:
            columnLayout
        case .compactAffordance:
            compactAffordanceLayout
        }
    }

    private var columnLayout: some View {
        VStack(alignment: .leading, spacing: 0) {
            HaneulchiSectionHeader(title: "Sessions", count: rows.isEmpty ? nil : rows.count)

            VStack(alignment: .leading, spacing: 0) {
                ForEach(rows) { row in
                    sessionRow(row)
                }
            }
        }
        .frame(width: columnWidth, alignment: .topLeading)
        .background(HaneulchiChrome.Surface.recess)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }

    private var compactAffordanceLayout: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
            HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.xs) {
                Text("Current Session")
                    .font(HaneulchiTypography.sectionHeading)
                    .foregroundStyle(HaneulchiChrome.Label.primary)

                if !rows.isEmpty {
                    Text("\(rows.count)")
                        .font(HaneulchiTypography.compactMeta)
                        .tracking(HaneulchiTypography.Tracking.metaModerate)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                        .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
                        .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
                        .background(HaneulchiChrome.Surface.foundation)
                        .clipShape(
                            RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.pill),
                        )
                }

                Spacer()
            }

            if let primaryRow {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                    HStack(alignment: .top, spacing: HaneulchiMetrics.Spacing.xs) {
                        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                            Text(primaryRow.title)
                                .font(HaneulchiTypography.systemLabel)
                                .tracking(HaneulchiTypography.Tracking.labelWide)
                                .foregroundStyle(HaneulchiChrome.Label.primary)
                                .lineLimit(2)

                            Text(primaryRow.summary)
                                .font(HaneulchiTypography.compactMeta)
                                .tracking(HaneulchiTypography.Tracking.metaModerate)
                                .foregroundStyle(HaneulchiChrome.Label.secondary)
                                .lineLimit(2)
                        }

                        Spacer(minLength: HaneulchiMetrics.Spacing.sm)

                        if let signal = primaryRow.signal {
                            HaneulchiStatusBadge(
                                state: signal.badgeState,
                                label: signal.label,
                            )
                        }
                    }

                    HStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.xs) {
                        if let branch = primaryRow.branch {
                            Text(branch)
                                .font(HaneulchiTypography.compactMeta)
                                .tracking(HaneulchiTypography.Tracking.metaModerate)
                                .foregroundStyle(HaneulchiChrome.Label.muted)
                                .lineLimit(1)
                        }

                        if primaryRow.unreadCount > 0 {
                            Text("\(primaryRow.unreadCount) unread")
                                .font(HaneulchiTypography.compactMeta)
                                .tracking(HaneulchiTypography.Tracking.metaModerate)
                                .foregroundStyle(HaneulchiChrome.State.warning)
                        }

                        Spacer()

                        Button(primaryActionTitle(for: primaryRow)) {
                            onAction(.jumpToSession(primaryRow.sessionID))
                        }
                        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                    }
                }
                .padding(HaneulchiMetrics.Padding.compact)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(HaneulchiChrome.Surface.base)
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
                .paneAttentionDecoration(
                    hasAttention: primaryRow.signal?.tone == .strong,
                    hasUnread: primaryRow.unreadCount > 0,
                )
            } else {
                Text("No active sessions.")
                    .font(HaneulchiTypography.bodySmall)
                    .foregroundStyle(HaneulchiChrome.Label.secondary)
            }
        }
        .padding(HaneulchiMetrics.Padding.compact)
        .background(HaneulchiChrome.Surface.recess)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }

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
                                        : HaneulchiChrome.Label.secondary,
                                )
                                .lineLimit(row.isFocused ? 2 : 1)

                            Spacer()
                        }

                        HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                            if let signal = row.signal {
                                HaneulchiStatusBadge(
                                    state: signal.badgeState,
                                    label: signal.label,
                                )
                            }

                            if let branch = row.branch {
                                Text(branch)
                                    .font(HaneulchiTypography.compactMeta)
                                    .tracking(HaneulchiTypography.Tracking.metaModerate)
                                    .foregroundStyle(HaneulchiChrome.Label.muted)
                                    .lineLimit(1)
                            }

                            Spacer()
                        }

                        Text(row.summary)
                            .font(HaneulchiTypography.compactMeta)
                            .tracking(HaneulchiTypography.Tracking.metaModerate)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                            .lineLimit(row.isFocused ? 2 : 1)
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
                        : HaneulchiChrome.Surface.recess,
                )
            }
            .buttonStyle(.plain)
            .paneAttentionDecoration(
                hasAttention: row.signal?.tone == .strong,
                hasUnread: row.unreadCount > 0,
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

    private var primaryRow: Row? {
        rows.first(where: \.isFocused) ?? rows.first
    }

    private func primaryActionTitle(for row: Row) -> String {
        row.showsManualContinueCTA ? "Manual Continue" : "Open Session"
    }

    nonisolated static func rows(from snapshot: AppShellSnapshot) -> [Row] {
        snapshot.sessions.map { session in
            let isFocused = session.focusState == .focused || snapshot.app
                .focusedSessionID == session.sessionID
            return Row(
                sessionID: session.sessionID,
                title: session.title,
                summary: session.latestSummary ?? session.currentDirectory ?? session.title,
                branch: session.branch,
                unreadCount: session.unreadCount,
                signal: SessionSignalPresentation.from(session: session, isFocused: isFocused),
                isFocused: isFocused,
                showsManualContinueCTA: session.canTakeover || session
                    .manualControlState == .takeover,
            )
        }
    }
}
