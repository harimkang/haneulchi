import Foundation

struct ProjectFileIndex: Sendable {
    struct Entry: Equatable, Identifiable, Sendable {
        let relativePath: String
        let absolutePath: String

        var id: String {
            absolutePath
        }
    }

    private let ignoredDirectoryNames: Set<String> = [
        ".git",
        ".build",
        "node_modules",
        "DerivedData",
    ]
    private let onEntryVisit: @Sendable () async throws -> Void

    init(
        onEntryVisit: @escaping @Sendable () async throws -> Void = {},
    ) {
        self.onEntryVisit = onEntryVisit
    }

    func index(rootPath: String, limit: Int = 300) async throws -> [Entry] {
        try await withThrowingTaskGroup(of: [Entry].self) { group in
            group.addTask(priority: .userInitiated) {
                try await scan(rootPath: rootPath, limit: limit)
            }

            let results = try await group.next() ?? []
            group.cancelAll()
            return results
        }
    }

    private func scan(rootPath: String, limit: Int) async throws -> [Entry] {
        let rootURL = URL(fileURLWithPath: rootPath, isDirectory: true)
        let canonicalRootPath = rootURL.resolvingSymlinksInPath().path
        guard FileManager.default.fileExists(atPath: rootURL.path) else {
            return []
        }

        let enumerator = FileManager.default.enumerator(
            at: rootURL,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsPackageDescendants],
        )

        var results: [Entry] = []
        while let next = enumerator?.nextObject() as? URL, results.count < limit {
            try Task.checkCancellation()
            try await onEntryVisit()
            try Task.checkCancellation()

            let values = try next.resourceValues(forKeys: [.isDirectoryKey])
            if ignoredDirectoryNames.contains(next.lastPathComponent),
               values.isDirectory == true
            {
                enumerator?.skipDescendants()
                continue
            }

            guard values.isDirectory != true else {
                continue
            }

            let canonicalPath = next.resolvingSymlinksInPath().path
            let relativePath: String = if canonicalPath.hasPrefix(canonicalRootPath + "/") {
                String(canonicalPath.dropFirst(canonicalRootPath.count + 1))
            } else {
                next.lastPathComponent
            }
            guard !relativePath.isEmpty else {
                continue
            }

            results.append(.init(relativePath: relativePath, absolutePath: next.path))
        }

        return results
            .sorted {
                $0.relativePath
                    .localizedCaseInsensitiveCompare($1.relativePath) == .orderedAscending
            }
    }
}
