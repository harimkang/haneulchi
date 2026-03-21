import Foundation

struct TerminalSessionRestoreStore: Sendable {
    let save: @Sendable ([TerminalRestoreBundle]) throws -> Void
    let load: @Sendable () throws -> [TerminalRestoreBundle]

    static var inMemory: Self {
        let storage = InMemoryTerminalSessionRestoreStorage()
        return Self(
            save: { bundles in
                try storage.save(bundles)
            },
            load: {
                try storage.load()
            }
        )
    }

    static func fileBacked(fileURL: URL) -> Self {
        Self(
            save: { bundles in
                let encoder = JSONEncoder()
                let data = try encoder.encode(bundles)
                try FileManager.default.createDirectory(
                    at: fileURL.deletingLastPathComponent(),
                    withIntermediateDirectories: true
                )
                try data.write(to: fileURL, options: .atomic)
            },
            load: {
                guard FileManager.default.fileExists(atPath: fileURL.path) else {
                    return []
                }

                let data = try Data(contentsOf: fileURL)
                return try JSONDecoder().decode([TerminalRestoreBundle].self, from: data)
            }
        )
    }

    static var liveDefault: Self {
        fileBacked(fileURL: defaultFileURL)
    }

    private static var defaultFileURL: URL {
        let applicationSupport =
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Library/Application Support")

        return applicationSupport
            .appendingPathComponent("Haneulchi", isDirectory: true)
            .appendingPathComponent("terminal-restore-bundles.json")
    }
}

private final class InMemoryTerminalSessionRestoreStorage: @unchecked Sendable {
    private let lock = NSLock()
    private var bundles: [TerminalRestoreBundle] = []

    func save(_ bundles: [TerminalRestoreBundle]) throws {
        lock.lock()
        defer { lock.unlock() }
        self.bundles = bundles
    }

    func load() throws -> [TerminalRestoreBundle] {
        lock.lock()
        defer { lock.unlock() }
        return bundles
    }
}
