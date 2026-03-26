import SwiftUI

struct InspectorPanelView: View {
    @Binding var workspaceState: ProjectFocusWorkspaceState
    let snapshot: AppShellSnapshot?
    let onAction: (AppShellAction) -> Void
    let controlStyle: InspectorSectionControlStyle

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HaneulchiSectionHeader(title: "Inspector")

            sectionPicker

            ScrollView {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                    sectionContent
                        .font(HaneulchiTypography.body)
                        .foregroundStyle(HaneulchiChrome.Label.secondary)
                }
                .padding(HaneulchiMetrics.Padding.compact)
            }

            Spacer()
        }
        .frame(
            maxWidth: .infinity,
            maxHeight: .infinity,
            alignment: .topLeading,
        )
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }

    @ViewBuilder
    private var sectionContent: some View {
        switch workspaceState.activeInspectorSection {
        case .commentary:
            if let focusedSession = Self.focusedSession(from: snapshot),
               focusedSession.providerID != nil || focusedSession.latestCommentary != nil
            {
                HaneulchiPanel {
                    AdapterWatchSummaryView(session: focusedSession)
                }
            } else {
                HaneulchiPanel {
                    Text(snapshot?.attention.first?.headline ?? "No commentary selected.")
                        .font(HaneulchiTypography.body)
                        .foregroundStyle(HaneulchiChrome.Label.secondary)
                }
            }
        case .task:
            HaneulchiPanel {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                    Text(Self.focusedSession(from: snapshot)?.taskID ?? "No linked task.")
                        .font(HaneulchiTypography.body)
                        .foregroundStyle(HaneulchiChrome.Label.secondary)
                    if Self.focusedSession(from: snapshot)?.taskID != nil {
                        Button {
                            onAction(.presentTaskContextDrawer)
                        } label: {
                            Label("Open Task Context", systemImage: "sidebar.right")
                        }
                        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                    }
                }
            }
        case .activity:
            HaneulchiPanel {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                    Text(Self.focusedSession(from: snapshot)?.latestSummary ?? "No activity yet.")
                        .font(HaneulchiTypography.body)
                        .foregroundStyle(HaneulchiChrome.Label.secondary)
                    if let retry = snapshot?.retryQueue.first {
                        Text("retry: attempt \(retry.attempt) · due \(retry.dueAt ?? "pending")")
                            .font(HaneulchiTypography.compactMeta)
                            .tracking(HaneulchiTypography.Tracking.metaModerate)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                    }
                }
            }
        case .evidence:
            HaneulchiPanel {
                Text("Evidence surface reserved for Sprint 2.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
            }
        case .git:
            HaneulchiPanel {
                Text(Self.focusedSession(from: snapshot)?.branch ?? "No branch information.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.secondary)
            }
        case .diff:
            HaneulchiPanel {
                Text("Diff surface reserved for Sprint 2.")
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
            }
        case .quickActions:
            HaneulchiPanel {
                Button {
                    onAction(.presentQuickDispatch(.projectFocus))
                } label: {
                    Label(
                        "Open Quick Dispatch",
                        systemImage: HaneulchiChromeAction.dispatch.symbolName,
                    )
                }
                .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
            }
        }
    }

    nonisolated static func focusedSession(from snapshot: AppShellSnapshot?) -> AppShellSnapshot
        .SessionSummary?
    {
        guard let snapshot else {
            return nil
        }

        return snapshot.sessions.first(where: {
            $0.focusState == .focused || snapshot.app.focusedSessionID == $0.sessionID
        }) ?? snapshot.sessions.first
    }

    @ViewBuilder
    private var sectionPicker: some View {
        switch controlStyle {
        case .segmented:
            Picker("Section", selection: $workspaceState.activeInspectorSection) {
                ForEach(InspectorSection.allCases, id: \.self) { section in
                    Text(section.controlTitle).tag(section)
                }
            }
            .pickerStyle(.segmented)
            .padding(HaneulchiMetrics.Padding.compact)
        case .compactScroll:
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                    ForEach(InspectorSection.allCases, id: \.self) { section in
                        Button {
                            workspaceState.activeInspectorSection = section
                        } label: {
                            Text(section.controlTitle)
                        }
                        .buttonStyle(
                            HaneulchiButtonStyle(
                                variant: section == workspaceState.activeInspectorSection
                                    ? .primary
                                    : .secondary,
                            ),
                        )
                    }
                }
                .padding(.horizontal, HaneulchiMetrics.Padding.compact)
                .padding(.vertical, HaneulchiMetrics.Spacing.xs)
            }
        }
    }
}

private extension InspectorSection {
    var controlTitle: String {
        switch self {
        case .commentary:
            "Commentary"
        case .task:
            "Task"
        case .activity:
            "Activity"
        case .evidence:
            "Evidence"
        case .git:
            "Git"
        case .diff:
            "Diff"
        case .quickActions:
            "Actions"
        }
    }
}
