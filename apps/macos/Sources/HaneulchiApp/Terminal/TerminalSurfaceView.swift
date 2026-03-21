import SwiftUI

struct TerminalSurfaceConfiguration: Equatable, Identifiable, Sendable {
    let id: String
    let title: String
    let fixtureName: String?

    static let projectFocusDemo = Self(
        id: "project-focus-demo",
        title: "Hosted Terminal",
        fixtureName: "hello-world.ansi"
    )
}

struct TerminalSurfaceView: View {
    let configuration: TerminalSurfaceConfiguration
    private let state: TerminalSurfaceState

    init(
        configuration: TerminalSurfaceConfiguration,
        controller: TerminalTranscriptController = TerminalTranscriptController()
    ) {
        self.configuration = configuration
        self.state = controller.bootstrap(fixtureName: configuration.fixtureName)
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
