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
        args: [],
        currentDirectory: nil,
        geometry: .defaultShell
    )
}

struct TerminalSessionSnapshot: Codable, Equatable, Sendable {
    let sessionID: String
    let launch: TerminalSessionLaunchRequest
    let geometry: TerminalGridSize
    let running: Bool
    let exitCode: Int?

    enum CodingKeys: String, CodingKey {
        case sessionID = "session_id"
        case launch
        case geometry
        case running
        case exitCode = "exit_code"
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
    ) {
        self.sessionID = sessionID
        self.launch = launch
        self.geometry = geometry
        self.running = running
        self.exitCode = exitCode
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let geometry = try container.decode(TerminalGridSize.self, forKey: .geometry)
        let launchPayload = try container.decode(LaunchPayload.self, forKey: .launch)

        self.sessionID = try container.decode(String.self, forKey: .sessionID)
        self.geometry = geometry
        self.running = try container.decode(Bool.self, forKey: .running)
        self.exitCode = try container.decodeIfPresent(Int.self, forKey: .exitCode)
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
    @Published private(set) var restorePoint = TerminalRestoreBundle.demo
    @Published private(set) var sessionSnapshot: TerminalSessionSnapshot?

    init(bridge: CoreBridge = .live) {
        self.bridge = bridge
    }

    func start(_ request: TerminalSessionLaunchRequest) async throws {
        status = .starting
        let snapshot = try bridge.spawnSession(request)
        sessionID = snapshot.sessionID
        sessionSnapshot = snapshot
        geometry = snapshot.geometry
        restorePoint = TerminalRestoreBundle(launch: request, geometry: snapshot.geometry)
        status = snapshot.running ? .running : .terminated(exitCode: snapshot.exitCode)
        try await refresh()
        startDrainLoop()
    }

    func restore(_ bundle: TerminalRestoreBundle) async throws {
        try await start(bundle.launch)
        if bundle.geometry != geometry {
            try resize(bundle.geometry)
        }
        restorePoint = bundle
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
        guard let sessionID else {
            return
        }

        try bridge.resizeSession(sessionID, geometry)
        self.geometry = geometry
        restorePoint = TerminalRestoreBundle(launch: restorePoint.launch, geometry: geometry)
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
}
