import SwiftUI

struct WorktreeInventoryRowView: View {
    let row: WorktreeInventoryViewModel.Row
    let onAction: (AppShellAction) -> Void

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
                    Text(ByteCountFormatter.string(fromByteCount: Int64(sizeBytes), countStyle: .file))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            VStack(alignment: .leading, spacing: 4) {
                    // Primary navigation actions
                    if row.canOpenFinder || row.canOpenSession || row.canOpenTask {
                        HStack(spacing: 8) {
                            if row.canOpenFinder {
                                Button("Open in Finder") { onAction(.openInventoryFinder(path: row.path)) }
                                    .buttonStyle(.bordered).controlSize(.small)
                            }
                            if row.canOpenSession {
                                if row.disposition == .inUse {
                                    Button("Open Session") {
                                        onAction(.openInventorySession(taskID: row.taskID, worktreeId: row.worktreeId))
                                    }
                                    .buttonStyle(.bordered).controlSize(.small)
                                } else if row.disposition == .recoverable {
                                    Button("Open Session") {
                                        onAction(.triggerRecovery(issueCode: "recover:\(row.worktreeId)"))
                                    }
                                    .buttonStyle(.bordered).controlSize(.small)
                                }
                            }
                            if row.canOpenTask {
                                Button("Open Task") { onAction(.openInventoryTask(taskID: row.taskID)) }
                                    .buttonStyle(.bordered).controlSize(.small)
                            }
                        }
                    }
                    // Lifecycle management actions
                    // Lifecycle actions are suppressed for .inUse rows (canRecover, canDelete,
                    // canPin are all false). In-use worktrees need no lifecycle management.
                    if row.canRecover || row.canDelete || row.canPin {
                        HStack(spacing: 8) {
                            if row.canRecover {
                                Button("Recover") { onAction(.triggerRecovery(issueCode: "recover:\(row.worktreeId)")) }
                                    .buttonStyle(.bordered).controlSize(.small)
                            }
                            if row.canDelete {
                                Button("Clean") { onAction(.triggerRecovery(issueCode: "clean:\(row.worktreeId)")) }
                                    .buttonStyle(.bordered).controlSize(.small)
                                    .foregroundStyle(.red)
                            }
                            if row.canPin {
                                Button(row.isPinned ? "Unpin" : "Pin") {
                                    onAction(.triggerRecovery(issueCode: "pin:\(row.worktreeId):\((!row.isPinned).description)"))
                                }
                                    .buttonStyle(.bordered).controlSize(.small)
                            }
                        }
                    }
                }
        }
        .padding(.vertical, 4)
    }

    @ViewBuilder
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
        case .inUse: return "In Use"
        case .recoverable: return "Recoverable"
        case .safeToDelete: return "Safe to Delete"
        case .stale: return "Stale"
        }
    }

    var badgeForeground: Color {
        switch self {
        case .inUse: return .white
        case .recoverable: return .white
        case .safeToDelete: return .white
        case .stale: return .primary
        }
    }

    var badgeBackground: Color {
        switch self {
        case .inUse: return .blue
        case .recoverable: return .orange
        case .safeToDelete: return .green
        case .stale: return Color(.systemGray).opacity(0.5)
        }
    }
}
