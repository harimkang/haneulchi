import SwiftUI

struct TerminalDeckView: View {
    struct Model: Equatable, Sendable {
        let layout: TerminalDeckLayout
        let showsSplitControls: Bool

        static let demo = Self(
            layout: .singleDemo,
            showsSplitControls: false,
        )

        static let runtimeDemo = Self(
            layout: .singleLiveDemo,
            showsSplitControls: false,
        )

        static func restored(_ bundle: TerminalRestoreBundle) -> Self {
            Self(
                layout: .singleLive(bundle),
                showsSplitControls: false,
            )
        }
    }

    let model: Model
    let signalPresentation: SessionSignalPresentation?
    let onQuickDispatch: (() -> Void)?
    let onSessionReady: ((String) -> Void)?
    @State private var layout: TerminalDeckLayout
    @StateObject private var deckCoordinator = TerminalDeckCoordinator()
    @State private var keyMonitor: Any?

    // `WF-02` reserves seams for Session Stack and Inspector outside the central deck.
    private let reservedSessionStackWidth: CGFloat = 220
    private let reservedInspectorWidth: CGFloat = 320

    init(
        model: Model,
        signalPresentation: SessionSignalPresentation? = nil,
        onQuickDispatch: (() -> Void)? = nil,
        onSessionReady: ((String) -> Void)? = nil,
    ) {
        self.model = model
        self.signalPresentation = signalPresentation
        self.onQuickDispatch = onQuickDispatch
        self.onSessionReady = onSessionReady
        _layout = State(initialValue: model.layout)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
            render(node: layout.root)
        }
        .padding(HaneulchiMetrics.Padding.compact)
        .background(HaneulchiChrome.Surface.recess)
        .onAppear {
            deckCoordinator.updateFocusedPane(layout.focusedPaneID)

            guard keyMonitor == nil else {
                return
            }

            keyMonitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown]) { event in
                deckCoordinator.handleKeyDown(event) ? nil : event
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
            AnyView(paneView(pane))
        case let .split(_, axis, _, first, second):
            switch axis {
            case .horizontal:
                AnyView(HSplitView {
                    render(node: first)
                    render(node: second)
                })
            case .vertical:
                AnyView(VSplitView {
                    render(node: first)
                    render(node: second)
                })
            }
        }
    }

    private func paneView(_ pane: TerminalPaneModel) -> some View {
        let isFocused = pane.id == layout.focusedPaneID

        return VStack(alignment: .leading, spacing: 0) {
            // Session header — compact height, dense chip row
            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                Text(pane.surface.title)
                    .font(HaneulchiTypography.systemLabel)
                    .tracking(HaneulchiTypography.Tracking.labelWide)
                    .foregroundStyle(
                        isFocused
                            ? HaneulchiChrome.Label.primary
                            : HaneulchiChrome.Label.secondary,
                    )
                if isFocused, let signalPresentation {
                    HaneulchiStatusBadge(
                        state: signalPresentation.badgeState,
                        label: signalPresentation.label,
                    )
                }
                Spacer()
                actionStrip(for: pane, isFocused: isFocused)
            }
            .frame(minHeight: HaneulchiMetrics.Target.compact)
            .padding(.horizontal, HaneulchiMetrics.Padding.compact)
            .background(
                isFocused
                    ? HaneulchiChrome.Surface.foundation
                    : HaneulchiChrome.Surface.recess,
            )
            .contentShape(Rectangle())
            .onTapGesture {
                layout.focusPane(pane.id)
                deckCoordinator.updateFocusedPane(pane.id)
                deckCoordinator.focusPane(pane.id)
            }

            // Terminal content area — recess/foundation tone only, no chrome styling on glyphs
            TerminalSurfaceView(
                configuration: pane.surface,
                paneID: pane.id,
                deckCoordinator: deckCoordinator,
                isFocused: isFocused,
                onSessionReady: onSessionReady,
            )
            .background(HaneulchiChrome.Surface.recess)
        }
        .background(HaneulchiChrome.Surface.recess)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
        .overlay(
            RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium)
                .strokeBorder(
                    isFocused
                        ? HaneulchiChrome.Gradient.primaryEnd.opacity(0.45)
                        : Color.clear,
                    lineWidth: 1,
                ),
        )
        .paneAttentionDecoration(
            hasAttention: isFocused && signalPresentation?.tone == .strong,
            hasUnread: isFocused && signalPresentation?.tone == .weak,
        )
        .frame(
            minWidth: max(320, reservedSessionStackWidth),
            maxWidth: .infinity,
            maxHeight: .infinity,
            alignment: .topLeading,
        )
        .accessibilityLabel("terminal-pane-\(pane.id)-inspector-\(Int(reservedInspectorWidth))")
    }

    @ViewBuilder
    private func actionStrip(for pane: TerminalPaneModel, isFocused: Bool) -> some View {
        if isFocused, pane.surface.isLive {
            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                HaneulchiIconButton(action: .focusPane, tone: .tertiary, size: 28) {
                    deckCoordinator.focusPane(pane.id)
                }
                HaneulchiIconButton(action: .find, tone: .tertiary, size: 28) {
                    deckCoordinator.showFind(in: pane.id)
                }
                HaneulchiIconButton(action: .paste, tone: .tertiary, size: 28) {
                    deckCoordinator.pasteClipboard(in: pane.id)
                }
                if let onQuickDispatch {
                    HaneulchiIconButton(action: .dispatch, tone: .tertiary, size: 28) {
                        onQuickDispatch()
                    }
                }
                HaneulchiIconButton(action: .splitHorizontal, tone: .tertiary, size: 28) {
                    splitFocusedPane(axis: .horizontal)
                }
                HaneulchiIconButton(action: .splitVertical, tone: .tertiary, size: 28) {
                    splitFocusedPane(axis: .vertical)
                }
            }
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
