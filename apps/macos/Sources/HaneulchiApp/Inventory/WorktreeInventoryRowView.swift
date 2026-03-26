import SwiftUI

enum WorktreeInventoryRowActionLayout: Equatable, Sendable {
    case inline
    case stacked
}

struct WorktreeInventoryRowPresentation: Equatable, Sendable {
    let actionLayout: WorktreeInventoryRowActionLayout

    init(viewportClass: HaneulchiViewportClass) {
        actionLayout = viewportClass == .compact ? .stacked : .inline
    }
}

struct WorktreeInventoryRowView: View {
    let row: WorktreeInventoryViewModel.Row
    let onAction: (AppShellAction) -> Void
    @Environment(\.viewportContext) private var viewportContext

    private var presentation: WorktreeInventoryRowPresentation {
        .init(viewportClass: viewportContext.viewportClass)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(row.path)
                        .font(.system(.body, design: .monospaced))
                        .lineLimit(1)
                        .truncationMode(.middle)

                    if let branch = row.branch {
                        Text(branch)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                Spacer()

                dispositionBadge

                if let sizeBytes = row.sizeBytes {
                    Text(ByteCountFormatter.string(
                        fromByteCount: Int64(sizeBytes),
                        countStyle: .file,
                    ))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                }
            }

            VStack(alignment: .leading, spacing: 4) {
                // Primary navigation actions
                if row.canOpenFinder || row.canOpenSession || row.canOpenTask {
                    actionGroup {
                        primaryActions
                    }
                }
                // Lifecycle management actions
                // Lifecycle actions are suppressed for .inUse rows (canRecover, canDelete,
                // canPin are all false). In-use worktrees need no lifecycle management.
                if row.canRecover || row.canDelete || row.canPin {
                    actionGroup {
                        lifecycleActions
                    }
                }
            }
        }
        .padding(.vertical, 4)
    }

    @ViewBuilder
    private func actionGroup(@ViewBuilder content: () -> some View) -> some View {
        switch presentation.actionLayout {
        case .inline:
            ViewThatFits(in: .horizontal) {
                HStack(spacing: 8) {
                    content()
                }

                VStack(alignment: .leading, spacing: 8) {
                    content()
                }
            }
        case .stacked:
            VStack(alignment: .leading, spacing: 8) {
                content()
            }
        }
    }

    private var primaryActions: some View {
        Group {
            if row.canOpenFinder {
                inventoryButton("Open in Finder") {
                    onAction(.openInventoryFinder(path: row.path))
                }
            }
            if row.canOpenSession {
                if row.disposition == .inUse {
                    inventoryButton("Open Session") {
                        onAction(.openInventorySession(
                            taskID: row.taskID,
                            worktreeId: row.worktreeId,
                        ))
                    }
                } else if row.disposition == .recoverable {
                    inventoryButton("Open Session") {
                        onAction(.triggerRecovery(issueCode: "recover:\(row.worktreeId)"))
                    }
                }
            }
            if row.canOpenTask {
                inventoryButton("Open Task") {
                    onAction(.openInventoryTask(taskID: row.taskID))
                }
            }
        }
    }

    private var lifecycleActions: some View {
        Group {
            if row.canRecover {
                inventoryButton("Recover") {
                    onAction(.triggerRecovery(issueCode: "recover:\(row.worktreeId)"))
                }
            }
            if row.canDelete {
                inventoryButton("Clean", foregroundStyle: .red) {
                    onAction(.triggerRecovery(issueCode: "clean:\(row.worktreeId)"))
                }
            }
            if row.canPin {
                inventoryButton(row.isPinned ? "Unpin" : "Pin") {
                    onAction(
                        .triggerRecovery(
                            issueCode: "pin:\(row.worktreeId):\((!row.isPinned).description)",
                        ),
                    )
                }
            }
        }
    }

    @ViewBuilder
    private func inventoryButton(
        _ title: String,
        foregroundStyle: Color? = nil,
        action: @escaping () -> Void,
    ) -> some View {
        if let foregroundStyle {
            Button(title, action: action)
                .buttonStyle(.bordered)
                .controlSize(.small)
                .foregroundStyle(foregroundStyle)
        } else {
            Button(title, action: action)
                .buttonStyle(.bordered)
                .controlSize(.small)
        }
    }

    private var dispositionBadge: some View {
        Text(row.disposition.badgeLabel)
            .font(.caption2.weight(.semibold))
            .foregroundStyle(row.disposition.badgeForeground)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(row.disposition.badgeBackground, in: Capsule())
    }
}

private extension WorktreeInventoryViewModel.Disposition {
    var badgeLabel: String {
        switch self {
        case .inUse: "In Use"
        case .recoverable: "Recoverable"
        case .safeToDelete: "Safe to Delete"
        case .stale: "Stale"
        }
    }

    var badgeForeground: Color {
        switch self {
        case .inUse: .white
        case .recoverable: .white
        case .safeToDelete: .white
        case .stale: .primary
        }
    }

    var badgeBackground: Color {
        switch self {
        case .inUse: .blue
        case .recoverable: .orange
        case .safeToDelete: .green
        case .stale: Color(.systemGray).opacity(0.5)
        }
    }
}
