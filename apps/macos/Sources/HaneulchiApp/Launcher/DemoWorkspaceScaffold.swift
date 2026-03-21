import Foundation

struct DemoWorkspaceScaffold {
    let materialize: () throws -> LauncherProject

    static func fileBacked(
        baseDirectory: URL,
        fileManager: FileManager = .default
    ) -> Self {
        Self(
            materialize: {
                let demoRoot = baseDirectory
                    .appendingPathComponent("Haneulchi", isDirectory: true)
                    .appendingPathComponent("Demo Workspace", isDirectory: true)

                try fileManager.createDirectory(
                    at: demoRoot,
                    withIntermediateDirectories: true
                )

                try writeIfMissing(
                    DemoWorkspaceTemplate.readme,
                    to: demoRoot.appendingPathComponent("README.md"),
                    fileManager: fileManager
                )
                try writeIfMissing(
                    DemoWorkspaceTemplate.workflow,
                    to: demoRoot.appendingPathComponent("WORKFLOW.md"),
                    fileManager: fileManager
                )

                return LauncherProject(
                    projectID: "proj_demo_workspace",
                    name: "Demo Workspace",
                    rootPath: demoRoot.path,
                    lastOpenedAt: .now
                )
            }
        )
    }

    static var liveDefault: Self {
        let applicationSupport =
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Library/Application Support")
        return fileBacked(baseDirectory: applicationSupport)
    }
}

private enum DemoWorkspaceTemplate {
    static let readme = """
    # Demo Workspace

    This sample workspace exists to validate the first-run launcher flow.

    - Use `Continue with Generic Shell` after the readiness check.
    - Use `pwd` and `ls` to confirm the selected project root.
    """

    static let workflow = """
    ---
    name: Demo Workspace Workflow
    review:
      checklist:
        - Confirm the launcher selected the expected project root.
    workspace:
      strategy: worktree
      base_root: .
    ---

    Focus on the requested task and keep the launcher flow easy to verify.
    """
}

private func writeIfMissing(
    _ contents: String,
    to url: URL,
    fileManager: FileManager
) throws {
    guard !fileManager.fileExists(atPath: url.path) else {
        return
    }

    guard let data = contents.data(using: .utf8) else {
        throw CocoaError(.fileWriteInapplicableStringEncoding)
    }

    try data.write(to: url, options: .atomic)
}
