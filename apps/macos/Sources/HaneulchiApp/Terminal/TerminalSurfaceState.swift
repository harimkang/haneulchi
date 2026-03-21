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
            return nil
        case .live:
            return nil
        case .empty:
            return "No transcript fixture configured for this hosted surface."
        case let .degraded(_, message):
            return message
        case let .failed(message):
            return message
        }
    }
}
