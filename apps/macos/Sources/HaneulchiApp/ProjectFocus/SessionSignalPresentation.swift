import SwiftUI

struct SessionSignalPresentation: Equatable, Sendable {
    enum Tone: String, Equatable, Sendable {
        case strong
        case weak
    }

    let tone: Tone
    let label: String
    let badgeState: HaneulchiStatusBadge.State

    static func from(
        session: AppShellSnapshot.SessionSummary,
        isFocused: Bool,
    ) -> SessionSignalPresentation? {
        if session.canTakeover || session.manualControlState == .takeover {
            return .init(tone: .strong, label: "manual takeover", badgeState: .manualTakeover)
        }

        switch session.runtimeState {
        case .waitingInput:
            return .init(tone: .strong, label: "awaiting input", badgeState: .waitingInput)
        case .reviewReady:
            return .init(tone: .strong, label: "review ready", badgeState: .reviewReady)
        case .error, .blocked:
            return .init(
                tone: .strong,
                label: session.runtimeState.rawValue.replacingOccurrences(of: "_", with: " "),
                badgeState: .blocked,
            )
        default:
            break
        }

        if session.unreadCount > 0 {
            return .init(
                tone: .weak,
                label: "\(session.unreadCount) unread",
                badgeState: .waitingInput,
            )
        }

        if isFocused, session.dispatchState == .dispatchable {
            return .init(tone: .weak, label: "dispatchable", badgeState: .idle)
        }

        return nil
    }

    var foregroundStyle: Color {
        switch tone {
        case .strong:
            HaneulchiChrome.Colors.ready
        case .weak:
            HaneulchiChrome.Colors.warning
        }
    }

    var backgroundStyle: Color {
        switch tone {
        case .strong:
            HaneulchiChrome.Colors.ready.opacity(0.14)
        case .weak:
            HaneulchiChrome.Colors.warning.opacity(0.14)
        }
    }
}
