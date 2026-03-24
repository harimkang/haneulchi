import SwiftUI

struct TerminalSurfaceConfiguration: Equatable, Identifiable, Sendable {
    enum Source: Equatable, Sendable {
        case fixture(String?)
        case live(TerminalRestoreBundle)
    }

    let id: String
    let title: String
    let source: Source

    static let projectFocusDemo = Self(
        id: "project-focus-demo",
        title: "Hosted Terminal",
        source: .fixture("hello-world.ansi")
    )

    static let projectFocusLiveDemo = Self(
        id: "project-focus-live-demo",
        title: "Hosted Terminal",
        source: .live(.demo)
    )

    static func liveSurface(id: String, bundle: TerminalRestoreBundle) -> Self {
        Self(id: id, title: "Hosted Terminal", source: .live(bundle))
    }

    var fixtureName: String? {
        guard case let .fixture(name) = source else {
            return nil
        }

        return name
    }

    var liveBundle: TerminalRestoreBundle? {
        guard case let .live(bundle) = source else {
            return nil
        }

        return bundle
    }

    var isLive: Bool {
        liveBundle != nil
    }

    func duplicated(withID id: String) -> Self {
        Self(id: id, title: title, source: source)
    }
}

struct TerminalSurfaceView: View {
    let configuration: TerminalSurfaceConfiguration
    private let state: TerminalSurfaceState
    @StateObject private var liveController: TerminalSessionController
    private let liveBundle: TerminalRestoreBundle?
    private let restoreStore: TerminalSessionRestoreStore
    private let paneID: String?
    private let deckCoordinator: TerminalDeckCoordinator?
    private let isFocused: Bool
    private let onSessionReady: ((String) -> Void)?
    @State private var reportedSessionID: String?

    init(
        configuration: TerminalSurfaceConfiguration,
        controller: TerminalTranscriptController = TerminalTranscriptController(),
        liveController: @autoclosure @escaping () -> TerminalSessionController = TerminalSessionController(),
        restoreStore: TerminalSessionRestoreStore = .liveDefault,
        paneID: String? = nil,
        deckCoordinator: TerminalDeckCoordinator? = nil,
        isFocused: Bool = false,
        onSessionReady: ((String) -> Void)? = nil
    ) {
        self.configuration = configuration
        self.liveBundle = configuration.liveBundle
        self.restoreStore = restoreStore
        self.paneID = paneID
        self.deckCoordinator = deckCoordinator
        self.isFocused = isFocused
        self.onSessionReady = onSessionReady
        self.state = if configuration.isLive {
            controller.bootstrapLive()
        } else {
            controller.bootstrap(fixtureName: configuration.fixtureName)
        }
        _liveController = StateObject(wrappedValue: liveController())
    }

    var body: some View {
        let resolvedState = state.resolvedFailure(liveController.failureMessage)

        VStack(alignment: .leading, spacing: 12) {
            ZStack(alignment: .bottomLeading) {
                RoundedRectangle(cornerRadius: 14)
                    .fill(Color(nsColor: .windowBackgroundColor))

                Group {
                    if let transcript = resolvedState.transcript {
                        TerminalRendererHost(
                            transcript: transcript,
                            onHostHandleReady: registerHostHandle
                        )
                    } else if liveBundle != nil, resolvedState.kind == .ready {
                        TerminalRendererHost.live(
                            controller: liveController,
                            onHostHandleReady: registerHostHandle
                        )
                            .task {
                                guard let liveBundle, liveController.status == .idle else {
                                    return
                                }

                                do {
                                    try await liveController.restore(liveBundle)
                                } catch {
                                    // The controller publishes an operator-visible failure state.
                                }
                            }
                            .onReceive(liveController.$restorePoint) { bundle in
                                try? restoreStore.save([bundle])
                            }
                            .onReceive(liveController.$sessionSnapshot) { snapshot in
                                guard
                                    let snapshot,
                                    snapshot.running,
                                    reportedSessionID != snapshot.sessionID
                                else {
                                    return
                                }
                                reportedSessionID = snapshot.sessionID
                                onSessionReady?(snapshot.sessionID)
                            }
                    } else {
                        statusView(for: resolvedState)
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 14))
            }
            .frame(minHeight: 320)
            .overlay(
                RoundedRectangle(cornerRadius: 14)
                    .strokeBorder(borderColor(for: resolvedState), lineWidth: 1)
            )
        }
        .onChange(of: isFocused) { _, focused in
            guard focused, let paneID else {
                return
            }

            deckCoordinator?.focusPane(paneID)
        }
    }

    private func statusView(for state: TerminalSurfaceState) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(statusTitle(for: state))
                .font(.title3.weight(.semibold))
            Text(state.message ?? "")
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .padding(20)
    }

    private func statusTitle(for state: TerminalSurfaceState) -> String {
        switch state.kind {
        case .ready:
            return configuration.title
        case .empty:
            return "Empty Surface"
        case .degraded:
            return "Degraded Surface"
        case .failed:
            return "Failed Surface"
        }
    }

    private func borderColor(for state: TerminalSurfaceState) -> Color {
        switch state.kind {
        case .ready:
            return .secondary.opacity(0.25)
        case .empty:
            return .secondary.opacity(0.25)
        case .degraded:
            return .orange.opacity(0.6)
        case .failed:
            return .red.opacity(0.65)
        }
    }

    @MainActor
    private func registerHostHandle(_ handle: TerminalHostHandle) {
        guard let paneID else {
            return
        }

        deckCoordinator?.register(handle, for: paneID)
    }
}
