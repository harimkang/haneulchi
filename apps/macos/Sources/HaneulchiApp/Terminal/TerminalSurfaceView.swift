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

    init(
        configuration: TerminalSurfaceConfiguration,
        controller: TerminalTranscriptController = TerminalTranscriptController(),
        liveController: @autoclosure @escaping () -> TerminalSessionController = TerminalSessionController(),
        restoreStore: TerminalSessionRestoreStore = .liveDefault
    ) {
        self.configuration = configuration
        self.liveBundle = configuration.liveBundle
        self.restoreStore = restoreStore
        self.state = if configuration.isLive {
            controller.bootstrapLive()
        } else {
            controller.bootstrap(fixtureName: configuration.fixtureName)
        }
        _liveController = StateObject(wrappedValue: liveController())
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(configuration.title)
                .font(.headline)

            ZStack(alignment: .bottomLeading) {
                RoundedRectangle(cornerRadius: 14)
                    .fill(Color(nsColor: .windowBackgroundColor))

                Group {
                    if let transcript = state.transcript {
                        TerminalRendererHost(transcript: transcript)
                    } else if liveBundle != nil, state.kind == .ready {
                        TerminalRendererHost.live(controller: liveController)
                            .task {
                                guard let liveBundle, liveController.status == .idle else {
                                    return
                                }

                                try? await liveController.restore(liveBundle)
                            }
                            .onReceive(liveController.$restorePoint) { bundle in
                                try? restoreStore.save([bundle])
                            }
                    } else {
                        statusView
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 14))
            }
            .frame(minHeight: 320)
            .overlay(
                RoundedRectangle(cornerRadius: 14)
                    .strokeBorder(borderColor, lineWidth: 1)
            )
        }
    }

    private var statusView: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(statusTitle)
                .font(.title3.weight(.semibold))
            Text(state.message ?? "")
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .padding(20)
    }

    private var statusTitle: String {
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

    private var borderColor: Color {
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
}
