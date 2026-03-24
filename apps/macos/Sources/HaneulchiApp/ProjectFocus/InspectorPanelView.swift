import SwiftUI

struct InspectorPanelView: View {
    @Binding var workspaceState: ProjectFocusWorkspaceState
    let snapshot: AppShellSnapshot?
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Inspector")
                .font(.headline)

            Picker("Section", selection: $workspaceState.activeInspectorSection) {
                Text("Commentary").tag(InspectorSection.commentary)
                Text("Task").tag(InspectorSection.task)
                Text("Activity").tag(InspectorSection.activity)
                Text("Evidence").tag(InspectorSection.evidence)
                Text("Git").tag(InspectorSection.git)
                Text("Diff").tag(InspectorSection.diff)
                Text("Quick Actions").tag(InspectorSection.quickActions)
            }
            .pickerStyle(.segmented)

            Group {
                switch workspaceState.activeInspectorSection {
                case .commentary:
                    if let focusedSession = Self.focusedSession(from: snapshot),
                       focusedSession.providerID != nil || focusedSession.latestCommentary != nil {
                        AdapterWatchSummaryView(session: focusedSession)
                    } else {
                        Text(snapshot?.attention.first?.headline ?? "No commentary selected.")
                    }
                case .task:
                    VStack(alignment: .leading, spacing: 8) {
                        Text(Self.focusedSession(from: snapshot)?.taskID ?? "No linked task.")
                        if Self.focusedSession(from: snapshot)?.taskID != nil {
                            Button("Open Task Context") {
                                onAction(.presentTaskContextDrawer)
                            }
                            .buttonStyle(.bordered)
                            .font(.caption.weight(.semibold))
                        }
                    }
                case .activity:
                    VStack(alignment: .leading, spacing: 6) {
                        Text(Self.focusedSession(from: snapshot)?.latestSummary ?? "No activity yet.")
                        if let retry = snapshot?.retryQueue.first {
                            Text("retry: attempt \(retry.attempt) · due \(retry.dueAt ?? "pending")")
                        }
                    }
                case .evidence:
                    Text("Evidence surface reserved for Sprint 2.")
                case .git:
                    Text(Self.focusedSession(from: snapshot)?.branch ?? "No branch information.")
                case .diff:
                    Text("Diff surface reserved for Sprint 2.")
                case .quickActions:
                    Button("Open Quick Dispatch") {
                        onAction(.presentQuickDispatch(.projectFocus))
                    }
                    .buttonStyle(.bordered)
                }
            }
            .font(HaneulchiTypography.body)
            .foregroundStyle(.secondary)

            Spacer()
        }
        .padding(16)
        .frame(width: 320, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.surfaceRaised)
    }

    nonisolated static func focusedSession(from snapshot: AppShellSnapshot?) -> AppShellSnapshot.SessionSummary? {
        guard let snapshot else {
            return nil
        }

        return snapshot.sessions.first(where: {
            $0.focusState == .focused || snapshot.app.focusedSessionID == $0.sessionID
        }) ?? snapshot.sessions.first
    }
}
