import Foundation
import Testing
@testable import HaneulchiApp

@Test("recent project store keeps the last-opened project first and deduplicates by root path")
func projectLauncherStoreKeepsMostRecentProjectFirst() throws {
    let store = ProjectLauncherStore.inMemory
    let alpha = LauncherProject(
        projectID: "proj_alpha",
        name: "alpha",
        rootPath: "/tmp/alpha",
        lastOpenedAt: .now
    )
    let beta = LauncherProject(
        projectID: "proj_beta",
        name: "beta",
        rootPath: "/tmp/beta",
        lastOpenedAt: .now.addingTimeInterval(5)
    )

    try store.recordOpen(alpha)
    try store.recordOpen(beta)
    try store.recordOpen(alpha)

    let recent = try store.loadRecentProjects()
    #expect(recent.map(\.rootPath) == [alpha.rootPath, beta.rootPath])
    #expect(recent.map(\.projectID) == [alpha.projectID, beta.projectID])
}
