import Foundation
import Testing
@testable import HaneulchiApp

private final class SendableBox<T>: @unchecked Sendable {
    var value: T

    init(_ value: T) {
        self.value = value
    }
}

private final class SessionTerminationRecorder: @unchecked Sendable {
    private let lock = NSLock()
    private(set) var terminatedSessionIDs: [String] = []

    func record(_ sessionID: String) {
        lock.lock()
        terminatedSessionIDs.append(sessionID)
        lock.unlock()
    }
}

@MainActor
@Test("live session controller drains terminal output from the Rust runtime")
func liveSessionControllerDrainsOutput() async throws {
    let bridge = CoreBridge.mockLiveSession(outputChunks: ["ready\n"])
    let controller = TerminalSessionController(bridge: bridge)

    try await controller.start(.defaultShell)

    #expect(controller.status == .running)
    #expect(controller.latestText.contains("ready"))
}

@MainActor
@Test("session controller injects resolved secret refs into the spawned launch request")
func liveSessionControllerInjectsResolvedSecretEnvironment() async throws {
    let capturedEnvironment = SendableBox<[String: String]>([:])
    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(
                rendererID: "swiftterm",
                transport: "ffi_c_abi",
                demoMode: false
            )
        },
        spawnSession: { request in
            capturedEnvironment.value = request.environment
            return TerminalSessionSnapshot(
                sessionID: "session-with-env",
                launch: request,
                geometry: request.geometry,
                running: true,
                exitCode: nil
            )
        },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in
            TerminalSessionSnapshot(
                sessionID: "session-with-env",
                launch: .defaultShell,
                geometry: .defaultShell,
                running: true,
                exitCode: nil
            )
        },
        resolveLaunchEnvironment: {
            [
                "OPENAI_API_KEY": "sk-live-test",
                "GITHUB_TOKEN": "gh-live-test",
            ]
        }
    )
    let controller = TerminalSessionController(bridge: bridge)

    try await controller.start(.defaultShell)

    #expect(capturedEnvironment.value["OPENAI_API_KEY"] == "sk-live-test")
    #expect(capturedEnvironment.value["GITHUB_TOKEN"] == "gh-live-test")
}

@MainActor
@Test("session controller preserves shell integration metadata from refreshed snapshots")
func liveSessionControllerPreservesShellMetadata() async throws {
    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(
                rendererID: "swiftterm",
                transport: "ffi_c_abi",
                demoMode: false
            )
        },
        spawnSession: { request in
            TerminalSessionSnapshot(
                sessionID: "session-shell-metadata",
                launch: request,
                geometry: request.geometry,
                running: true,
                exitCode: nil
            )
        },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { requestID in
            #expect(requestID == "session-shell-metadata")
            return TerminalSessionSnapshot(
                sessionID: "session-shell-metadata",
                launch: .defaultShell,
                geometry: .defaultShell,
                running: true,
                exitCode: nil,
                shellMetadata: .init(
                    currentDirectory: "/tmp/demo",
                    lastCommand: "npm test",
                    lastExitCode: 17,
                    branch: "main"
                )
            )
        }
    )
    let controller = TerminalSessionController(bridge: bridge)

    try await controller.start(.defaultShell)

    #expect(controller.sessionSnapshot?.shellMetadata?.currentDirectory == "/tmp/demo")
    #expect(controller.sessionSnapshot?.shellMetadata?.lastCommand == "npm test")
    #expect(controller.sessionSnapshot?.shellMetadata?.lastExitCode == 17)
}

@MainActor
@Test("session controller exposes restore failure without pretending the session is running")
func liveSessionControllerExposesRestoreFailure() async {
    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(
                rendererID: "swiftterm",
                transport: "ffi_c_abi",
                demoMode: false
            )
        },
        spawnSession: { _ in
            throw CoreBridgeError.operationFailed("session_spawn_failed")
        },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in
            throw CoreBridgeError.operationFailed("session_not_found")
        }
    )
    let controller = TerminalSessionController(bridge: bridge)

    await #expect(throws: CoreBridgeError.operationFailed("session_spawn_failed")) {
        try await controller.restore(.demo)
    }

    #expect(controller.status == .failed)
    #expect(controller.failureMessage == "Hosted terminal could not start.")
}

@MainActor
@Test("controller terminates a spawned session if the first refresh fails")
func liveSessionControllerTerminatesSpawnedSessionWhenBootstrapRefreshFails() async {
    let recorder = SessionTerminationRecorder()
    let spawnedSessionID = "session-bootstrap"
    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(
                rendererID: "swiftterm",
                transport: "ffi_c_abi",
                demoMode: false
            )
        },
        spawnSession: { request in
            TerminalSessionSnapshot(
                sessionID: spawnedSessionID,
                launch: request,
                geometry: request.geometry,
                running: true,
                exitCode: nil
            )
        },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { sessionID in
            recorder.record(sessionID)
        },
        snapshotSession: { _ in
            throw CoreBridgeError.operationFailed("snapshot_failed")
        }
    )
    let controller = TerminalSessionController(bridge: bridge)

    await #expect(throws: CoreBridgeError.operationFailed("snapshot_failed")) {
        try await controller.start(.defaultShell)
    }

    #expect(recorder.terminatedSessionIDs == ["session-bootstrap"])
    #expect(controller.status == .failed)
}

@MainActor
@Test("failed restore keeps the previous restore point unchanged")
func failedRestoreDoesNotOverwriteRestorePoint() async throws {
    let attemptedBundle = TerminalRestoreBundle.genericShell(at: "/tmp/failed")
    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(
                rendererID: "swiftterm",
                transport: "ffi_c_abi",
                demoMode: false
            )
        },
        spawnSession: { request in
            TerminalSessionSnapshot(
                sessionID: "session-fail",
                launch: request,
                geometry: request.geometry,
                running: true,
                exitCode: nil
            )
        },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in
            throw CoreBridgeError.operationFailed("snapshot_failed")
        }
    )
    let controller = TerminalSessionController(bridge: bridge)

    await #expect(throws: CoreBridgeError.operationFailed("snapshot_failed")) {
        try await controller.restore(attemptedBundle)
    }

    #expect(controller.restorePoint == .demo)
}
