import Foundation

struct ProjectLauncherStore: Sendable {
    let loadRecentProjects: @Sendable () throws -> [LauncherProject]
    let recordOpen: @Sendable (LauncherProject) throws -> Void
    let loadLastSelectedProject: @Sendable () throws -> LauncherProject?
    let saveLastSelectedProject: @Sendable (LauncherProject?) throws -> Void

    static var inMemory: Self {
        let storage = InMemoryProjectLauncherStorage()
        return Self(
            loadRecentProjects: { try storage.loadRecentProjects() },
            recordOpen: { try storage.recordOpen($0) },
            loadLastSelectedProject: { try storage.loadLastSelectedProject() },
            saveLastSelectedProject: { try storage.saveLastSelectedProject($0) }
        )
    }

    static func fileBacked(fileURL: URL) -> Self {
        Self(
            loadRecentProjects: {
                try FileBackedProjectLauncherStorage(fileURL: fileURL).loadRecentProjects()
            },
            recordOpen: { project in
                let storage = FileBackedProjectLauncherStorage(fileURL: fileURL)
                try storage.recordOpen(project)
            },
            loadLastSelectedProject: {
                try FileBackedProjectLauncherStorage(fileURL: fileURL).loadLastSelectedProject()
            },
            saveLastSelectedProject: { project in
                let storage = FileBackedProjectLauncherStorage(fileURL: fileURL)
                try storage.saveLastSelectedProject(project)
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
            .appendingPathComponent("project-launcher-state.json")
    }
}

private struct ProjectLauncherPersistence: Codable {
    var recentProjects: [LauncherProject]
    var lastSelectedProject: LauncherProject?

    static let empty = Self(recentProjects: [], lastSelectedProject: nil)
}

private final class InMemoryProjectLauncherStorage: @unchecked Sendable {
    private let lock = NSLock()
    private var state = ProjectLauncherPersistence.empty

    func loadRecentProjects() throws -> [LauncherProject] {
        lock.lock()
        defer { lock.unlock() }
        return state.recentProjects
    }

    func recordOpen(_ project: LauncherProject) throws {
        lock.lock()
        defer { lock.unlock() }
        let refreshed = LauncherProject(
            projectID: project.projectID,
            name: project.name,
            rootPath: project.rootPath,
            lastOpenedAt: Date()
        )
        state.recentProjects.removeAll { $0.rootPath == refreshed.rootPath }
        state.recentProjects.insert(refreshed, at: 0)
    }

    func loadLastSelectedProject() throws -> LauncherProject? {
        lock.lock()
        defer { lock.unlock() }
        return state.lastSelectedProject
    }

    func saveLastSelectedProject(_ project: LauncherProject?) throws {
        lock.lock()
        defer { lock.unlock() }
        state.lastSelectedProject = project
    }
}

private struct FileBackedProjectLauncherStorage {
    let fileURL: URL

    func loadRecentProjects() throws -> [LauncherProject] {
        try loadState().recentProjects
    }

    func recordOpen(_ project: LauncherProject) throws {
        var state = try loadState()
        let refreshed = LauncherProject(
            projectID: project.projectID,
            name: project.name,
            rootPath: project.rootPath,
            lastOpenedAt: Date()
        )
        state.recentProjects.removeAll { $0.rootPath == refreshed.rootPath }
        state.recentProjects.insert(refreshed, at: 0)
        try saveState(state)
    }

    func loadLastSelectedProject() throws -> LauncherProject? {
        try loadState().lastSelectedProject
    }

    func saveLastSelectedProject(_ project: LauncherProject?) throws {
        var state = try loadState()
        state.lastSelectedProject = project
        try saveState(state)
    }

    private func loadState() throws -> ProjectLauncherPersistence {
        guard FileManager.default.fileExists(atPath: fileURL.path) else {
            return .empty
        }

        let data = try Data(contentsOf: fileURL)
        return try JSONDecoder().decode(ProjectLauncherPersistence.self, from: data)
    }

    private func saveState(_ state: ProjectLauncherPersistence) throws {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try encoder.encode(state)
        try FileManager.default.createDirectory(
            at: fileURL.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try data.write(to: fileURL, options: .atomic)
    }
}
