import SwiftUI

struct TerminalDeckView: View {
    struct Model: Equatable, Sendable {
        let layout: TerminalDeckLayout
        let showsSplitControls: Bool

        static let demo = Self(
            layout: .singleDemo,
            showsSplitControls: false
        )

        static let runtimeDemo = Self(
            layout: .singleLiveDemo,
            showsSplitControls: false
        )

        static func restored(_ bundle: TerminalRestoreBundle) -> Self {
            Self(
                layout: .singleLive(bundle),
                showsSplitControls: false
            )
        }
    }

    let model: Model

    // `WF-02` reserves seams for Session Stack and Inspector outside the central deck.
    private let reservedSessionStackWidth: CGFloat = 220
    private let reservedInspectorWidth: CGFloat = 320

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Project Focus")
                .font(.largeTitle)
                .bold()

            Text("Central deck keeps focus on terminal work while preserving room for Session Stack and Inspector.")
                .foregroundStyle(.secondary)

            render(node: model.layout.root)
        }
    }

    private func render(node: TerminalDeckNode) -> AnyView {
        switch node {
        case let .pane(pane):
            return AnyView(paneView(pane))
        case let .split(_, axis, _, first, second):
            switch axis {
            case .horizontal:
                return AnyView(HSplitView {
                    render(node: first)
                    render(node: second)
                })
            case .vertical:
                return AnyView(VSplitView {
                    render(node: first)
                    render(node: second)
                })
            }
        }
    }

    private func paneView(_ pane: TerminalPaneModel) -> some View {
        let isFocused = pane.id == model.layout.focusedPaneID

        return VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Text(pane.surface.title)
                    .font(.headline)
                Spacer()
                if model.showsSplitControls {
                    Text("Split")
                        .font(.caption.weight(.semibold))
                        .padding(.horizontal, 10)
                        .padding(.vertical, 4)
                        .background(Color.primary.opacity(0.08))
                        .clipShape(Capsule())
                }
            }

            TerminalSurfaceView(configuration: pane.surface)
        }
        .padding(16)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(
            RoundedRectangle(cornerRadius: 18)
                .fill(isFocused ? Color(nsColor: .controlBackgroundColor) : Color(nsColor: .windowBackgroundColor))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 18)
                .strokeBorder(isFocused ? Color.accentColor.opacity(0.55) : Color.clear, lineWidth: 1)
        )
        .frame(
            minWidth: max(320, reservedSessionStackWidth),
            maxWidth: .infinity,
            maxHeight: .infinity,
            alignment: .topLeading
        )
        .padding(.trailing, 0)
        .padding(.leading, 0)
        .padding(.bottom, 0)
        .accessibilityLabel("terminal-pane-\(pane.id)-inspector-\(Int(reservedInspectorWidth))")
    }
}
