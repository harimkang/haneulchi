import Foundation

struct LauncherProject: Codable, Equatable, Identifiable, Sendable {
    let projectID: String
    let name: String
    let rootPath: String
    let lastOpenedAt: Date

    var id: String {
        projectID
    }
}
