import Foundation

struct SystemProcessClient: Sendable {
    enum Response: Equatable, Sendable {
        case success(String)
        case failure(String)
    }

    let detectedShellPath: @Sendable () -> String?
    let run: @Sendable (String, String?) async throws -> Response

    static func mock(shellPath: String?, commands: [String: Response]) -> Self {
        Self(
            detectedShellPath: { shellPath },
            run: { command, detectedShellPath in
                if let detectedShellPath,
                   let response = commands["\(command) [shell:\(detectedShellPath)]"]
                {
                    return response
                }

                return commands[command] ?? .failure("command_not_stubbed")
            },
        )
    }

    static let live = Self(
        detectedShellPath: {
            guard let shellPath = ProcessInfo.processInfo.environment["SHELL"],
                  !shellPath.isEmpty
            else {
                return nil
            }

            return shellPath
        },
        run: { command, shellPath in
            let process = Process()
            let output = Pipe()
            let error = Pipe()
            process.executableURL = URL(fileURLWithPath: shellPath ?? "/bin/zsh")
            process.arguments = ["-lc", command]
            process.standardOutput = output
            process.standardError = error
            try process.run()
            process.waitUntilExit()

            let stdout = String(
                decoding: output.fileHandleForReading.readDataToEndOfFile(),
                as: UTF8.self,
            )
            let stderr = String(
                decoding: error.fileHandleForReading.readDataToEndOfFile(),
                as: UTF8.self,
            )

            if process.terminationStatus == 0 {
                return .success(stdout)
            }

            return .failure(stderr.isEmpty ? "command_failed" : stderr
                .trimmingCharacters(in: .whitespacesAndNewlines))
        },
    )
}
