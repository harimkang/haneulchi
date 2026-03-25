import Foundation

struct TerminalReplay: Equatable, Sendable {
    let backend: TerminalBackendDescriptor
    let transcript: String
}

enum TerminalSurfaceState: Equatable, Sendable {
    enum Kind: Equatable, Sendable {
        case ready
        case empty
        case degraded
        case failed
    }

    case ready(TerminalReplay)
    case live(TerminalBackendDescriptor)
    case empty(TerminalBackendDescriptor)
    case degraded(TerminalBackendDescriptor, message: String)
    case failed(message: String)

    var kind: Kind {
        switch self {
        case .ready:
            .ready
        case .live:
            .ready
        case .empty:
            .empty
        case .degraded:
            .degraded
        case .failed:
            .failed
        }
    }

    var transcript: String? {
        guard case let .ready(replay) = self else {
            return nil
        }

        return replay.transcript
    }

    var message: String? {
        switch self {
        case .ready:
            nil
        case .live:
            nil
        case .empty:
            "No transcript fixture configured for this hosted surface."
        case let .degraded(_, message):
            message
        case let .failed(message):
            message
        }
    }

    func resolvedFailure(_ message: String?) -> Self {
        guard let message else {
            return self
        }

        return .failed(message: message)
    }
}
