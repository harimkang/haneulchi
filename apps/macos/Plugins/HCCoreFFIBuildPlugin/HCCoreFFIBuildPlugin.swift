import Foundation
import PackagePlugin

@main
struct HCCoreFFIBuildPlugin: BuildToolPlugin {
    func createBuildCommands(context: PluginContext, target _: Target) throws -> [Command] {
        let packageDirectory = context.package.directoryURL
        let workspaceRoot = packageDirectory
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        let script = packageDirectory
            .appending(path: "Plugins/HCCoreFFIBuildPlugin/build-hc-ffi.sh")
        let outputDirectory = context.pluginWorkDirectoryURL
            .appending(path: "hc-ffi-build", directoryHint: .isDirectory)

        return [
            .prebuildCommand(
                displayName: "Building hc_ffi and generating terminal fixtures",
                executable: URL(filePath: "/bin/zsh"),
                arguments: [
                    "-l",
                    script.path(percentEncoded: false),
                    workspaceRoot.path(percentEncoded: false),
                    outputDirectory.path(percentEncoded: false),
                ],
                outputFilesDirectory: outputDirectory,
            ),
        ]
    }
}
