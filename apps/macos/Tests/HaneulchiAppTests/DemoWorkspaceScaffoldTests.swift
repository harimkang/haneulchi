import Foundation
import Testing
@testable import HaneulchiApp

@Test("demo workspace scaffold creates a stable project root with seed files")
func demoWorkspaceScaffoldCreatesStableProjectRoot() throws {
    let root = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent(UUID().uuidString)
    let scaffold = DemoWorkspaceScaffold.fileBacked(baseDirectory: root)

    let project = try scaffold.materialize()
    let projectRoot = URL(fileURLWithPath: project.rootPath, isDirectory: true)

    #expect(project.projectID == "proj_demo_workspace")
    #expect(project.name == "Demo Workspace")
    #expect(FileManager.default.fileExists(atPath: project.rootPath))
    #expect(FileManager.default.fileExists(atPath: projectRoot.appendingPathComponent("README.md").path))
    #expect(FileManager.default.fileExists(atPath: projectRoot.appendingPathComponent("WORKFLOW.md").path))
}

@Test("demo workspace scaffold is idempotent and preserves the same root path")
func demoWorkspaceScaffoldIsIdempotent() throws {
    let root = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent(UUID().uuidString)
    let scaffold = DemoWorkspaceScaffold.fileBacked(baseDirectory: root)

    let first = try scaffold.materialize()
    let second = try scaffold.materialize()

    #expect(first.projectID == second.projectID)
    #expect(first.rootPath == second.rootPath)
}
