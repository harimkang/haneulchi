import Foundation
import HCCoreFFI

enum CoreBridgeError: Error, Equatable {
    case invalidStringPayload
    case invalidRuntimeInfo
}

struct CoreBridge: Sendable {
    let runtimeInfo: @Sendable () throws -> TerminalBackendDescriptor

    static let live = Self(
        runtimeInfo: {
            let payload = hc_runtime_info_json()
            defer {
                hc_string_free(payload)
            }

            guard let pointer = payload.ptr else {
                throw CoreBridgeError.invalidStringPayload
            }

            guard let json = String(validatingCString: pointer) else {
                throw CoreBridgeError.invalidStringPayload
            }

            let data = Data(json.utf8)

            do {
                return try JSONDecoder().decode(TerminalBackendDescriptor.self, from: data)
            } catch {
                throw CoreBridgeError.invalidRuntimeInfo
            }
        }
    )
}
