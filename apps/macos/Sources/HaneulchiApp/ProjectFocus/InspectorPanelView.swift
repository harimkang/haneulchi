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
                    Text(snapshot?.attention.first?.headline ?? "No commentary selected.")
                case .task:
                    VStack(alignment: .leading, spacing: 8) {
                        Text(snapshot?.sessions.first?.taskID ?? "No linked task.")
                        if snapshot?.sessions.first?.taskID != nil {
                            Button("Open Task Context") {
                                onAction(.presentTaskContextDrawer)
                            }
                            .buttonStyle(.bordered)
                            .font(.caption.weight(.semibold))
                        }
                    }
                case .activity:
                    Text(snapshot?.sessions.first?.latestSummary ?? "No activity yet.")
                case .evidence:
                    Text("Evidence surface reserved for Sprint 2.")
                case .git:
                    Text(snapshot?.sessions.first?.branch ?? "No branch information.")
                case .diff:
                    Text("Diff surface reserved for Sprint 2.")
                case .quickActions:
                    Text("Quick actions surface reserved for Sprint 2.")
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
}
