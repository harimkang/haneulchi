import Foundation

struct WorktreeInventoryViewModel: Equatable, Sendable {
    enum Disposition: String, Equatable, Sendable, Codable {
        case inUse = "in_use"
        case recoverable
        case safeToDelete = "safe_to_delete"
        case stale
    }

    struct Row: Equatable, Identifiable, Sendable {
        let worktreeId: String
        let taskID: String
        let path: String
        let projectName: String
        let branch: String?
        let disposition: Disposition
        let isPinned: Bool
        let isDegraded: Bool
        let sizeBytes: Int?
        let lastAccessedAt: String?

        var id: String {
            worktreeId
        }

        /// Action gating — what actions are available for this row?
        var canDelete: Bool {
            disposition == .safeToDelete || disposition == .stale
        }

        var canRecover: Bool {
            disposition == .recoverable
        }

        var canPin: Bool {
            disposition != .inUse
        }

        var canOpenFinder: Bool {
            !path.isEmpty
        }

        /// `.inUse` rows navigate to the running session.
        /// `.recoverable` rows restore/recover the crashed session.
        var canOpenSession: Bool {
            (disposition == .inUse || disposition == .recoverable) && !path.isEmpty
        }

        var canOpenTask: Bool {
            !taskID.isEmpty
        }

        init(
            worktreeId: String,
            taskID: String = "",
            path: String,
            projectName: String,
            branch: String?,
            disposition: Disposition,
            isPinned: Bool,
            isDegraded: Bool,
            sizeBytes: Int?,
            lastAccessedAt: String?,
        ) {
            self.worktreeId = worktreeId
            self.taskID = taskID
            self.path = path
            self.projectName = projectName
            self.branch = branch
            self.disposition = disposition
            self.isPinned = isPinned
            self.isDegraded = isDegraded
            self.sizeBytes = sizeBytes
            self.lastAccessedAt = lastAccessedAt
        }
    }

    struct SummaryCard: Equatable, Sendable {
        let total: Int
        let inUse: Int
        let recoverable: Int
        let safeToDelete: Int
        let stale: Int
    }

    let summaryCard: SummaryCard
    // Groups in order: InUse, Recoverable, SafeToDelete, Stale
    let inUseRows: [Row]
    let recoverableRows: [Row]
    let safeToDeleteRows: [Row]
    let staleRows: [Row]

    init(rows: [Row]) {
        let inUse = rows.filter { $0.disposition == .inUse }
        let recoverable = rows.filter { $0.disposition == .recoverable }
        let safeToDelete = rows.filter { $0.disposition == .safeToDelete }
        let stale = rows.filter { $0.disposition == .stale }

        inUseRows = inUse
        recoverableRows = recoverable
        safeToDeleteRows = safeToDelete
        staleRows = stale

        summaryCard = SummaryCard(
            total: rows.count,
            inUse: inUse.count,
            recoverable: recoverable.count,
            safeToDelete: safeToDelete.count,
            stale: stale.count,
        )
    }
}
