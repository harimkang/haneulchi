import Foundation

struct AppShellPreferences: Codable, Equatable, Sendable {
    var lastActiveRoute: Route

    static let `default` = Self(lastActiveRoute: .projectFocus)
}

struct AppShellPreferencesStore: Sendable {
    let load: @Sendable () throws -> AppShellPreferences
    let save: @Sendable (AppShellPreferences) throws -> Void

    static var inMemory: Self {
        let storage = InMemoryAppShellPreferencesStorage()
        return Self(
            load: {
                try storage.load()
            },
            save: { preferences in
                try storage.save(preferences)
            },
        )
    }

    static func fileBacked(fileURL: URL) -> Self {
        Self(
            load: {
                guard FileManager.default.fileExists(atPath: fileURL.path) else {
                    return .default
                }

                let data = try Data(contentsOf: fileURL)
                return try JSONDecoder().decode(AppShellPreferences.self, from: data)
            },
            save: { preferences in
                let encoder = JSONEncoder()
                let data = try encoder.encode(preferences)
                try FileManager.default.createDirectory(
                    at: fileURL.deletingLastPathComponent(),
                    withIntermediateDirectories: true,
                )
                try data.write(to: fileURL, options: .atomic)
            },
        )
    }

    static var liveDefault: Self {
        fileBacked(fileURL: defaultFileURL)
    }

    private static var defaultFileURL: URL {
        let applicationSupport =
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
                ?? URL(fileURLWithPath: NSHomeDirectory())
                .appendingPathComponent("Library/Application Support")

        return applicationSupport
            .appendingPathComponent("Haneulchi", isDirectory: true)
            .appendingPathComponent("app-shell-preferences.json")
    }
}

private final class InMemoryAppShellPreferencesStorage: @unchecked Sendable {
    private let lock = NSLock()
    private var preferences = AppShellPreferences.default

    func load() throws -> AppShellPreferences {
        lock.lock()
        defer { lock.unlock() }
        return preferences
    }

    func save(_ preferences: AppShellPreferences) throws {
        lock.lock()
        defer { lock.unlock() }
        self.preferences = preferences
    }
}
