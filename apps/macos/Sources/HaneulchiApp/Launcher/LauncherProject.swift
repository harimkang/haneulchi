import Foundation

struct LauncherProject: Codable, Equatable, Identifiable, Sendable {
    let projectID: String
    let name: String
    let rootPath: String
    let lastOpenedAt: Date

    var id: String { projectID }

    init(projectID: String, name: String, rootPath: String, lastOpenedAt: Date) {
        self.projectID = projectID
        self.name = name
        self.rootPath = rootPath
        self.lastOpenedAt = lastOpenedAt
    }
}
