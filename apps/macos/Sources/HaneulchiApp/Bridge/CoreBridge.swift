import Foundation
import HCCoreFFI

enum CoreBridgeError: Error, Equatable {
    case invalidStringPayload
    case invalidBytesPayload
    case invalidRuntimeInfo
    case invalidSessionSnapshot
    case operationFailed(String)
}

struct CoreBridge: Sendable {
    let runtimeInfo: @Sendable () throws -> TerminalBackendDescriptor
    let spawnSession: @Sendable (TerminalSessionLaunchRequest) throws -> TerminalSessionSnapshot
    let drainSession: @Sendable (String) throws -> Data
    let writeSession: @Sendable (String, Data) throws -> Void
    let resizeSession: @Sendable (String, TerminalGridSize) throws -> Void
    let terminateSession: @Sendable (String) throws -> Void
    let snapshotSession: @Sendable (String) throws -> TerminalSessionSnapshot

    static let live = Self(
        runtimeInfo: {
            let data = try stringPayloadData(hc_runtime_info_json())

            guard let descriptor = try? JSONDecoder().decode(TerminalBackendDescriptor.self, from: data) else {
                throw CoreBridgeError.invalidRuntimeInfo
            }

            return descriptor
        },
        spawnSession: { request in
            let payload = try JSONEncoder().encode(request)
            let json = String(decoding: payload, as: UTF8.self)
            let config = try CStringBox(json)
            let responseData = try config.withPointer { pointer in
                try stringPayloadData(hc_terminal_session_spawn_json(pointer))
            }

            return try decodeSessionSnapshot(from: responseData)
        },
        drainSession: { sessionID in
            let session = try CStringBox(sessionID)
            let payload = session.withPointer { hc_terminal_session_drain($0) }
            defer { hc_bytes_free(payload) }

            guard let pointer = payload.ptr else {
                return Data()
            }

            return Data(bytes: pointer, count: payload.len)
        },
        writeSession: { sessionID, data in
            let session = try CStringBox(sessionID)
            let result = session.withPointer { pointer in
                data.withUnsafeBytes { rawBuffer in
                    hc_terminal_session_write(
                        pointer,
                        rawBuffer.baseAddress?.assumingMemoryBound(to: UInt8.self),
                        rawBuffer.count
                    )
                }
            }

            guard result == 0 else {
                throw CoreBridgeError.operationFailed("session_write_failed")
            }
        },
        resizeSession: { sessionID, geometry in
            let session = try CStringBox(sessionID)
            let result = session.withPointer {
                hc_terminal_session_resize(
                    $0,
                    UInt16(geometry.cols),
                    UInt16(geometry.rows)
                )
            }

            guard result == 0 else {
                throw CoreBridgeError.operationFailed("session_resize_failed")
            }
        },
        terminateSession: { sessionID in
            let session = try CStringBox(sessionID)
            let result = session.withPointer { hc_terminal_session_terminate($0) }

            guard result == 0 else {
                throw CoreBridgeError.operationFailed("session_terminate_failed")
            }
        },
        snapshotSession: { sessionID in
            let session = try CStringBox(sessionID)
            let payload = try session.withPointer { pointer in
                try stringPayloadData(hc_terminal_session_snapshot_json(pointer))
            }
            return try decodeSessionSnapshot(from: payload)
        }
    )

    static func mockLiveSession(outputChunks: [String]) -> Self {
        let state = MockLiveSessionState(outputChunks: outputChunks)

        return Self(
            runtimeInfo: {
                TerminalBackendDescriptor(
                    rendererID: "swiftterm",
                    transport: "ffi_c_abi",
                    demoMode: false
                )
            },
            spawnSession: { request in
                try state.spawn(request)
            },
            drainSession: { sessionID in
                try state.drain(sessionID: sessionID)
            },
            writeSession: { sessionID, data in
                try state.write(sessionID: sessionID, data: data)
            },
            resizeSession: { sessionID, geometry in
                try state.resize(sessionID: sessionID, geometry: geometry)
            },
            terminateSession: { sessionID in
                try state.terminate(sessionID: sessionID)
            },
            snapshotSession: { sessionID in
                try state.snapshot(sessionID: sessionID)
            }
        )
    }
}

private func stringPayloadData(_ payload: HcString) throws -> Data {
    defer {
        hc_string_free(payload)
    }

    guard let pointer = payload.ptr else {
        throw CoreBridgeError.invalidStringPayload
    }

    guard let json = String(validatingCString: pointer) else {
        throw CoreBridgeError.invalidStringPayload
    }

    if
        let data = json.data(using: .utf8),
        let response = try? JSONDecoder().decode(CoreBridgeErrorPayload.self, from: data),
        let error = response.error
    {
        throw CoreBridgeError.operationFailed(error)
    }

    return Data(json.utf8)
}

private func decodeSessionSnapshot(from data: Data) throws -> TerminalSessionSnapshot {
    guard let snapshot = try? JSONDecoder().decode(TerminalSessionSnapshot.self, from: data) else {
        throw CoreBridgeError.invalidSessionSnapshot
    }

    return snapshot
}

private struct CoreBridgeErrorPayload: Decodable {
    let error: String?
}

private struct CStringBox {
    private let storage: [CChar]

    init(_ string: String) throws {
        self.storage = Array(string.utf8CString)
    }

    func withPointer<T>(_ body: (UnsafePointer<CChar>) throws -> T) rethrows -> T {
        try storage.withUnsafeBufferPointer { buffer in
            try body(buffer.baseAddress!)
        }
    }
}

private final class MockLiveSessionState: @unchecked Sendable {
    private let lock = NSLock()
    private var snapshot: TerminalSessionSnapshot?
    private var pendingChunks: [Data]

    init(outputChunks: [String]) {
        self.pendingChunks = outputChunks.map { Data($0.utf8) }
    }

    func spawn(_ request: TerminalSessionLaunchRequest) throws -> TerminalSessionSnapshot {
        let snapshot = TerminalSessionSnapshot(
            sessionID: "mock-session",
            launch: request,
            geometry: request.geometry,
            running: true,
            exitCode: nil
        )

        lock.lock()
        self.snapshot = snapshot
        lock.unlock()

        return snapshot
    }

    func drain(sessionID: String) throws -> Data {
        lock.lock()
        defer { lock.unlock() }

        guard snapshot?.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        if pendingChunks.isEmpty {
            return Data()
        }

        return pendingChunks.removeFirst()
    }

    func write(sessionID: String, data: Data) throws {
        lock.lock()
        defer { lock.unlock() }

        guard snapshot?.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        pendingChunks.append(data)
    }

    func resize(sessionID: String, geometry: TerminalGridSize) throws {
        lock.lock()
        defer { lock.unlock() }

        guard var snapshot, snapshot.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        snapshot = TerminalSessionSnapshot(
            sessionID: snapshot.sessionID,
            launch: snapshot.launch,
            geometry: geometry,
            running: snapshot.running,
            exitCode: snapshot.exitCode
        )
        self.snapshot = snapshot
    }

    func terminate(sessionID: String) throws {
        lock.lock()
        defer { lock.unlock() }

        guard var snapshot, snapshot.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        snapshot = TerminalSessionSnapshot(
            sessionID: snapshot.sessionID,
            launch: snapshot.launch,
            geometry: snapshot.geometry,
            running: false,
            exitCode: 0
        )
        self.snapshot = snapshot
    }

    func snapshot(sessionID: String) throws -> TerminalSessionSnapshot {
        lock.lock()
        defer { lock.unlock() }

        guard let snapshot, snapshot.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        return snapshot
    }
}
