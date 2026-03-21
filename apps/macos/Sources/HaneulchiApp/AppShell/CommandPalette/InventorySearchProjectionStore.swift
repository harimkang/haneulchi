import Foundation

struct InventorySearchProjectionStore: Sendable {
    enum RowKind: String, Equatable, Sendable {
        case sharedRoot = "shared_root"
        case restoreRoot = "restore_root"
    }

    struct Row: Equatable, Identifiable, Sendable {
        let itemID: String
        let title: String
        let rootPath: String
        let kind: RowKind

        var id: String { itemID }
    }

    let restoreStore: TerminalSessionRestoreStore

    func load(selectedProjectRoot: String?) async throws -> [Row] {
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

            return .init(
                itemID: "inventory-\(index + 1)",
                title: title,
                rootPath: root,
                kind: root == selectedProjectRoot ? .sharedRoot : .restoreRoot
            )
        }
    }
}
