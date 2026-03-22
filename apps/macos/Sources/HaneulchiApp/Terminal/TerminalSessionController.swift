import Combine
import Foundation

struct TerminalGridSize: Codable, Equatable, Sendable {
    let cols: Int
    let rows: Int
}

struct TerminalSessionLaunchRequest: Codable, Equatable, Sendable {
    let program: String
    let args: [String]
    let currentDirectory: String?
    let geometry: TerminalGridSize

    enum CodingKeys: String, CodingKey {
        case program
        case args
        case currentDirectory = "current_directory"
        case geometry
    }

    static let defaultShell = Self(
        program: "/bin/zsh",
        args: defaultBootstrapArgs(for: "/bin/zsh"),
        currentDirectory: nil,
        geometry: .defaultShell
    )

    static func genericShell(at rootPath: String?) -> Self {
        Self(
            program: "/bin/zsh",
            args: defaultBootstrapArgs(for: "/bin/zsh"),
            currentDirectory: rootPath,
            geometry: .defaultShell
        )
    }

    private static func defaultBootstrapArgs(for program: String) -> [String] {
        let scriptPath = integrationScriptPath(for: program)
        return [
            "-lc",
            "source '\(scriptPath)'; exec \(program) -i",
        ]
    }

    private static func integrationScriptPath(for program: String) -> String {
        let scriptName = program.contains("bash") ? "haneulchi.bash" : "haneulchi.zsh"
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<6 {
            url.deleteLastPathComponent()
        }
        return url
            .appendingPathComponent("config/shell-integration")
            .appendingPathComponent(scriptName)
            .path
    }
}

struct ShellIntegrationMetadata: Codable, Equatable, Sendable {
    let currentDirectory: String?
    let lastCommand: String?
    let lastExitCode: Int?
    let branch: String?

    enum CodingKeys: String, CodingKey {
        case currentDirectory = "current_directory"
        case lastCommand = "last_command"
        case lastExitCode = "last_exit_code"
        case branch
    }
}

struct TerminalSessionSnapshot: Codable, Equatable, Sendable {
    let sessionID: String
    let launch: TerminalSessionLaunchRequest
    let geometry: TerminalGridSize
    let running: Bool
    let exitCode: Int?
    let shellMetadata: ShellIntegrationMetadata?

    enum CodingKeys: String, CodingKey {
        case sessionID = "session_id"
        case launch
        case geometry
        case running
        case exitCode = "exit_code"
        case shellMetadata = "shell_metadata"
    }

    private struct LaunchPayload: Decodable {
        let program: String
        let args: [String]
        let currentDirectory: String?
        let geometry: TerminalGridSize?

        enum CodingKeys: String, CodingKey {
            case program
            case args
            case currentDirectory = "current_directory"
            case geometry
        }
    }

    init(
        sessionID: String,
        launch: TerminalSessionLaunchRequest,
        geometry: TerminalGridSize,
        running: Bool,
        exitCode: Int?
        ,
        shellMetadata: ShellIntegrationMetadata? = nil
    ) {
        self.sessionID = sessionID
        self.launch = launch
        self.geometry = geometry
        self.running = running
        self.exitCode = exitCode
        self.shellMetadata = shellMetadata
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let geometry = try container.decode(TerminalGridSize.self, forKey: .geometry)
        let launchPayload = try container.decode(LaunchPayload.self, forKey: .launch)

        self.sessionID = try container.decode(String.self, forKey: .sessionID)
        self.geometry = geometry
        self.running = try container.decode(Bool.self, forKey: .running)
        self.exitCode = try container.decodeIfPresent(Int.self, forKey: .exitCode)
        self.shellMetadata = try container.decodeIfPresent(ShellIntegrationMetadata.self, forKey: .shellMetadata)
        self.launch = TerminalSessionLaunchRequest(
            program: launchPayload.program,
            args: launchPayload.args,
            currentDirectory: launchPayload.currentDirectory,
            geometry: launchPayload.geometry ?? geometry
        )
    }
}

struct TerminalRestoreBundle: Codable, Equatable, Sendable {
    let launch: TerminalSessionLaunchRequest
    let geometry: TerminalGridSize

    static let demo = Self(launch: .defaultShell, geometry: .defaultShell)

    static func genericShell(at rootPath: String?) -> Self {
        Self(launch: .genericShell(at: rootPath), geometry: .defaultShell)
    }
}

extension TerminalGridSize {
    static let defaultShell = Self(cols: 80, rows: 24)
}

@MainActor
final class TerminalSessionController: ObservableObject {
    enum Status: Equatable, Sendable {
        case idle
        case starting
        case running
        case terminated(exitCode: Int?)
        case failed
    }

    private let bridge: CoreBridge
    private var sessionID: String?
    private var drainTask: Task<Void, Never>?

    @Published private(set) var latestText = ""
    @Published private(set) var geometry = TerminalGridSize.defaultShell
    @Published private(set) var status: Status = .idle
    @Published private(set) var failureMessage: String?
    @Published private(set) var restorePoint = TerminalRestoreBundle.demo
    @Published private(set) var sessionSnapshot: TerminalSessionSnapshot?

    init(bridge: CoreBridge = .live) {
        self.bridge = bridge
    }

    func start(_ request: TerminalSessionLaunchRequest) async throws {
        status = .starting
        failureMessage = nil

        do {
            let snapshot = try bridge.spawnSession(request)
            sessionID = snapshot.sessionID
            sessionSnapshot = snapshot
            geometry = snapshot.geometry
            status = snapshot.running ? .running : .terminated(exitCode: snapshot.exitCode)
            try await refresh()
            restorePoint = TerminalRestoreBundle(launch: request, geometry: geometry)
            startDrainLoop()
        } catch {
            terminateCurrentSessionIfNeeded()
            recordFailure("Hosted terminal could not start.")
            throw error
        }
    }

    func restore(_ bundle: TerminalRestoreBundle) async throws {
        let previousRestorePoint = restorePoint

        do {
            try await start(bundle.launch)
            if bundle.geometry != geometry {
                try resize(bundle.geometry, updatesRestorePoint: false)
            }
            restorePoint = bundle
        } catch {
            restorePoint = previousRestorePoint

            if failureMessage == nil || sessionID != nil {
                terminateCurrentSessionIfNeeded()
                recordFailure("Hosted terminal could not start.")
            }
            throw error
        }
    }

    func refresh() async throws {
        guard let sessionID else {
            return
        }

        let drained = try bridge.drainSession(sessionID)
        if !drained.isEmpty {
            latestText += String(decoding: drained, as: UTF8.self)
        }

        let snapshot = try bridge.snapshotSession(sessionID)
        sessionSnapshot = snapshot
        geometry = snapshot.geometry
        status = snapshot.running ? .running : .terminated(exitCode: snapshot.exitCode)
    }

    func write(_ data: Data) throws {
        guard let sessionID else {
            return
        }

        try bridge.writeSession(sessionID, data)
    }

    func resize(_ geometry: TerminalGridSize) throws {
        try resize(geometry, updatesRestorePoint: true)
    }

    private func resize(_ geometry: TerminalGridSize, updatesRestorePoint: Bool) throws {
        guard let sessionID else {
            return
        }

        try bridge.resizeSession(sessionID, geometry)
        self.geometry = geometry
        if updatesRestorePoint {
            restorePoint = TerminalRestoreBundle(launch: restorePoint.launch, geometry: geometry)
        }
    }

    func terminate() throws {
        guard let sessionID else {
            return
        }

        try bridge.terminateSession(sessionID)
        status = .terminated(exitCode: sessionSnapshot?.exitCode)
        drainTask?.cancel()
    }

    deinit {
        drainTask?.cancel()
    }

    private func startDrainLoop() {
        drainTask?.cancel()
        drainTask = Task { @MainActor [weak self] in
            guard let self else {
                return
            }

            while !Task.isCancelled {
                try? await self.refresh()
                try? await Task.sleep(for: .milliseconds(33))
            }
        }
    }

    private func terminateCurrentSessionIfNeeded() {
        guard let sessionID else {
            return
        }

        try? bridge.terminateSession(sessionID)
    }

    private func recordFailure(_ message: String) {
        drainTask?.cancel()
        sessionID = nil
        sessionSnapshot = nil
        status = .failed
        failureMessage = message
    }
}
