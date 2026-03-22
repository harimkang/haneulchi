import SwiftUI

struct SessionStackView: View {
    struct Row: Equatable, Identifiable {
        let sessionID: String
        let title: String
        let summary: String
        let branch: String?
        let unreadCount: Int
        let isFocused: Bool
        let showsManualContinueCTA: Bool

        var id: String { sessionID }
    }

    let rows: [Row]
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Session Stack")
                .font(.headline)

            ForEach(rows) { row in
                VStack(alignment: .leading, spacing: 6) {
                    Button {
                        onAction(.jumpToSession(row.sessionID))
                    } label: {
                        VStack(alignment: .leading, spacing: 4) {
                            HStack {
                                Text(row.title)
                                    .font(.subheadline.weight(row.isFocused ? .semibold : .regular))
                                Spacer()
                                if row.unreadCount > 0 {
                                    Text("\(row.unreadCount)")
                                        .font(.caption.monospacedDigit())
                                        .padding(.horizontal, 6)
                                        .padding(.vertical, 2)
                                        .background(Color.secondary.opacity(0.15))
                                        .clipShape(Capsule())
                                }
                            }
                            if let branch = row.branch {
                                Text(branch)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                            Text(row.summary)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(12)
                        .background(row.isFocused ? HaneulchiChrome.Colors.surfaceRaised : HaneulchiChrome.Colors.secondaryPanel)
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                    }
                    .buttonStyle(.plain)

                    if row.showsManualContinueCTA {
                        Button("Open Session for Manual Continue") {
                            onAction(.jumpToSession(row.sessionID))
                        }
                        .buttonStyle(.bordered)
                        .font(.caption.weight(.semibold))
                    }
                }
            }
        }
        .padding(16)
        .frame(width: 240, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.secondaryPanel)
    }

    nonisolated static func rows(from snapshot: AppShellSnapshot) -> [Row] {
        snapshot.sessions.map { session in
            Row(
                sessionID: session.sessionID,
                title: session.title,
                summary: session.latestSummary ?? session.currentDirectory ?? session.title,
                branch: session.branch,
                unreadCount: session.unreadCount,
                isFocused: session.focusState == .focused || snapshot.app.focusedSessionID == session.sessionID,
                showsManualContinueCTA: session.canTakeover || session.manualControlState == .takeover
            )
        }
    }
}
