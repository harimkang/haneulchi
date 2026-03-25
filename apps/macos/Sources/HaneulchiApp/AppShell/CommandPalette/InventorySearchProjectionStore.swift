import Foundation

struct InventorySearchProjectionStore: Sendable {
    struct Row: Equatable, Identifiable, Sendable {
        let itemID: String
        let title: String
        let rootPath: String
        let disposition: String  // "in_use" | "recoverable" | "safe_to_delete" | "stale"

        var id: String { itemID }
    }

    /// When set, rows are loaded from the FFI inventory list for the selected project.
    let inventoryList: (@Sendable (String) throws -> [InventoryRowPayload])?

    /// Legacy restore-store fallback used when inventoryList is not provided.
    let restoreStore: TerminalSessionRestoreStore?

    init(inventoryList: @escaping @Sendable (String) throws -> [InventoryRowPayload]) {
        self.inventoryList = inventoryList
        self.restoreStore = nil
    }

    init(restoreStore: TerminalSessionRestoreStore) {
        self.restoreStore = restoreStore
        self.inventoryList = nil
    }

    func load(
        selectedProjectID: String? = nil,
        selectedProjectRoot: String? = nil
    ) async throws -> [Row] {
        if let inventoryList, let selectedProjectID, !selectedProjectID.isEmpty {
            let payloads = try inventoryList(selectedProjectID)
            return payloads.map { payload in
                let title = URL(fileURLWithPath: payload.path).lastPathComponent.isEmpty
                    ? payload.path
                    : URL(fileURLWithPath: payload.path).lastPathComponent
                return Row(
                    itemID: payload.worktreeId,
                    title: title,
                    rootPath: payload.path,
                    disposition: payload.disposition
                )
            }
        }

        // Legacy restore-store path
        guard let restoreStore else {
            return []
        }

        let restoreRoots = try restoreStore.load()
            .compactMap(\.launch.currentDirectory)

        var roots: [String] = []
        var seen = Set<String>()

        if let selectedProjectRoot, FileManager.default.fileExists(atPath: selectedProjectRoot) {
            roots.append(selectedProjectRoot)
            seen.insert(selectedProjectRoot)
        }

        for root in restoreRoots where FileManager.default.fileExists(atPath: root) && !seen.contains(root) {
            roots.append(root)
            seen.insert(root)
        }

        return roots.enumerated().map { index, root in
            let title = URL(fileURLWithPath: root).lastPathComponent.isEmpty
                ? root
                : URL(fileURLWithPath: root).lastPathComponent

            return Row(
                itemID: "inventory-\(index + 1)",
                title: title,
                rootPath: root,
                disposition: root == selectedProjectRoot ? "in_use" : "recoverable"
            )
        }
    }

    /// A mock store that returns a fixed set of rows — useful in tests.
    static let mock = InventorySearchProjectionStore(
        inventoryList: { _ in
            [
                InventoryRowPayload(
                    worktreeId: "wt-mock-1",
                    path: "/tmp/mock/worktree-1",
                    projectName: "mock-project",
                    branch: "main",
                    disposition: "in_use",
                    isPinned: false,
                    isDegraded: false,
                    sizeBytes: nil,
                    lastAccessedAt: nil
                )
            ]
        }
    )
}
