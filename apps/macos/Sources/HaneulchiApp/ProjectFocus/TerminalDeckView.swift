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
    let signalPresentation: SessionSignalPresentation?
    @State private var layout: TerminalDeckLayout
    @StateObject private var deckCoordinator = TerminalDeckCoordinator()
    @State private var keyMonitor: Any?

    // `WF-02` reserves seams for Session Stack and Inspector outside the central deck.
    private let reservedSessionStackWidth: CGFloat = 220
    private let reservedInspectorWidth: CGFloat = 320

    init(model: Model, signalPresentation: SessionSignalPresentation? = nil) {
        self.model = model
        self.signalPresentation = signalPresentation
        _layout = State(initialValue: model.layout)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            render(node: layout.root)
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .onAppear {
            deckCoordinator.updateFocusedPane(layout.focusedPaneID)

            guard keyMonitor == nil else {
                return
            }

            keyMonitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown]) { event in
                return deckCoordinator.handleKeyDown(event) ? nil : event
            }
        }
        .onDisappear {
            if let keyMonitor {
                NSEvent.removeMonitor(keyMonitor)
                self.keyMonitor = nil
            }
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
        let isFocused = pane.id == layout.focusedPaneID

        return VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Text(pane.surface.title)
                    .font(.headline)
                if isFocused, let signalPresentation {
                    Text(signalPresentation.label)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(signalPresentation.foregroundStyle)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(signalPresentation.backgroundStyle)
                        .clipShape(Capsule())
                }
                Spacer()
                actionStrip(for: pane, isFocused: isFocused)
            }
            .contentShape(Rectangle())
            .onTapGesture {
                layout.focusPane(pane.id)
                deckCoordinator.updateFocusedPane(pane.id)
                deckCoordinator.focusPane(pane.id)
            }

            TerminalSurfaceView(
                configuration: pane.surface,
                paneID: pane.id,
                deckCoordinator: deckCoordinator,
                isFocused: isFocused
            )
        }
        .padding(16)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(
            RoundedRectangle(cornerRadius: 18)
                .fill(isFocused ? HaneulchiChrome.Colors.surfaceRaised : HaneulchiChrome.Colors.surfaceBase)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 18)
                .strokeBorder(isFocused ? HaneulchiChrome.Colors.accent.opacity(0.45) : Color.clear, lineWidth: 1)
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

    @ViewBuilder
    private func actionStrip(for pane: TerminalPaneModel, isFocused: Bool) -> some View {
        if isFocused, pane.surface.isLive {
            HStack(spacing: 8) {
                Button("Focus") {
                    deckCoordinator.focusPane(pane.id)
                }
                Button("Find") {
                    deckCoordinator.showFind(in: pane.id)
                }
                Button("Paste") {
                    deckCoordinator.pasteClipboard(in: pane.id)
                }
                Button("Split H") {
                    splitFocusedPane(axis: .horizontal)
                }
                Button("Split V") {
                    splitFocusedPane(axis: .vertical)
                }
            }
            .font(.caption.weight(.semibold))
            .buttonStyle(.borderless)
            .foregroundStyle(.secondary)
        }
    }

    private func splitFocusedPane(axis: TerminalDeckAxis) {
        layout.splitFocusedPane(axis: axis)
        let focusedPaneID = layout.focusedPaneID
        deckCoordinator.updateFocusedPane(focusedPaneID)
        DispatchQueue.main.async {
            deckCoordinator.focusPane(focusedPaneID)
        }
    }
}
